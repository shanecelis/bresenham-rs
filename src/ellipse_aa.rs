//! Integer anti-aliased axis-aligned ellipse.
//!
//! The outline computes integer ellipse intersections and measures coverage
//! along each pixel's minor axis. The fill extends Vadillo's squared-distance
//! band by using the local gradient magnitude of the ellipse's implicit
//! equation.

use arraydeque::ArrayDeque;

#[cfg(feature = "fill")]
use crate::fill::{Fill, Plot, Span};
use crate::{CircleAa, Point, PointAa};

/// Anti-aliased axis-aligned ellipse given a center and radii.
///
/// Equal radii use [`CircleAa`] directly and therefore produce exactly the
/// same points, ordering, and coverage. Other ellipses are generated row by
/// row in linear perimeter time using integer square roots. Coverage is
/// calculated once in the first quadrant and reflected across both axes.
pub struct EllipseAa {
    #[cfg(feature = "fill")]
    center: Point,
    #[cfg(feature = "fill")]
    a: isize,
    #[cfg(feature = "fill")]
    b: isize,
    inner: EllipseAaInner,
}

// Keeping CircleAa inline preserves exact equal-radii output without requiring
// allocation in this no_std crate.
#[allow(clippy::large_enum_variant)]
enum EllipseAaInner {
    Circle(CircleAa),
    Ellipse(EllipseAaOutline),
    Horizontal { x: isize, x1: isize, y: isize },
    Vertical { x: isize, y: isize, y1: isize },
}

impl EllipseAa {
    /// Closed ellipse centered at `center`, with horizontal radius `a` and
    /// vertical radius `b`. Negative radii are treated as absolute values.
    pub fn new(center: Point, a: isize, b: isize) -> Self {
        let a = a.abs();
        let b = b.abs();
        let inner = if a == b {
            EllipseAaInner::Circle(CircleAa::new(center, a))
        } else if b == 0 {
            EllipseAaInner::Horizontal {
                x: center.0 - a,
                x1: center.0 + a,
                y: center.1,
            }
        } else if a == 0 {
            EllipseAaInner::Vertical {
                x: center.0,
                y: center.1 - b,
                y1: center.1 + b,
            }
        } else {
            EllipseAaInner::Ellipse(EllipseAaOutline::new(center, a, b))
        };
        Self {
            #[cfg(feature = "fill")]
            center,
            #[cfg(feature = "fill")]
            a,
            #[cfg(feature = "fill")]
            b,
            inner,
        }
    }
}

impl Iterator for EllipseAa {
    type Item = PointAa;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            EllipseAaInner::Circle(circle) => circle.next(),
            EllipseAaInner::Horizontal { x, x1, y } => {
                if *x > *x1 {
                    return None;
                }
                let point = ((*x, *y), 255);
                *x += 1;
                Some(point)
            }
            EllipseAaInner::Vertical { x, y, y1 } => {
                if *y > *y1 {
                    return None;
                }
                let point = ((*x, *y), 255);
                *y += 1;
                Some(point)
            }
            EllipseAaInner::Ellipse(ellipse) => ellipse.next(),
        }
    }
}

struct EllipseAaOutline {
    center: Point,
    a: isize,
    b: isize,
    a2: i128,
    b2: i128,
    target: i128,
    y: isize,
    boundary: isize,
    x: isize,
    x_end: isize,
    pending: ArrayDeque<PointAa, 4>,
}

impl EllipseAaOutline {
    fn new(center: Point, a: isize, b: isize) -> Self {
        let a2 = (a as i128) * (a as i128);
        let b2 = (b as i128) * (b as i128);
        let mut ellipse = Self {
            center,
            a,
            b,
            a2,
            b2,
            target: a2 * b2,
            y: 0,
            boundary: 0,
            x: 0,
            x_end: 0,
            pending: ArrayDeque::new(),
        };
        ellipse.enter_row();
        ellipse
    }

    fn implicit(&self, x: isize) -> i128 {
        let x = x as i128;
        let y = self.y as i128;
        self.b2 * x * x + self.a2 * y * y - self.target
    }

    fn alpha(&self, x: isize) -> u8 {
        let vertical = self.b2 * x as i128 <= self.a2 * self.y.abs() as i128;
        let z = 255u128;
        let (boundary, coordinate) = if vertical {
            let remainder = (self.a2 - (x as i128) * (x as i128)).max(0) as u128;
            let boundary = isqrt(remainder * self.b2 as u128 * z * z) / self.a as u128;
            (boundary, self.y.unsigned_abs() as u128 * z)
        } else {
            let y = self.y as i128;
            let remainder = (self.b2 - y * y).max(0) as u128;
            let boundary = isqrt(remainder * self.a2 as u128 * z * z) / self.b as u128;
            (boundary, x as u128 * z)
        };
        255u128.saturating_sub(boundary.abs_diff(coordinate).min(255)) as u8
    }

    fn enter_row(&mut self) {
        while self.boundary < self.a && self.implicit(self.boundary + 1) <= 0 {
            self.boundary += 1;
        }
        while self.boundary > 0 && self.implicit(self.boundary) > 0 {
            self.boundary -= 1;
        }

        let anchor = if self.alpha(self.boundary) > 0 {
            Some(self.boundary)
        } else if self.boundary < self.a && self.alpha(self.boundary + 1) > 0 {
            Some(self.boundary + 1)
        } else {
            None
        };
        let Some(anchor) = anchor else {
            self.x = 1;
            self.x_end = 0;
            return;
        };

        let mut first = anchor;
        while first > 0 && self.alpha(first - 1) > 0 {
            first -= 1;
        }
        let mut last = anchor;
        while last < self.a && self.alpha(last + 1) > 0 {
            last += 1;
        }
        self.x = first;
        self.x_end = last;
    }

    fn prepare_reflections(&mut self) {
        let x = self.x;
        let y = self.y;
        let alpha = self.alpha(x);
        let candidates = [
            (self.center.0 + x, self.center.1 + y),
            (self.center.0 - x, self.center.1 + y),
            (self.center.0 - x, self.center.1 - y),
            (self.center.0 + x, self.center.1 - y),
        ];
        'candidate: for point in candidates {
            for prior in self.pending.iter() {
                if prior.0 == point {
                    continue 'candidate;
                }
            }
            self.pending
                .push_back((point, alpha))
                .expect("EllipseAa outline pending overflow");
        }
        self.x += 1;
    }
}

impl Iterator for EllipseAaOutline {
    type Item = PointAa;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(point) = self.pending.pop_front() {
                return Some(point);
            }
            if self.x <= self.x_end {
                self.prepare_reflections();
                continue;
            }
            if self.y == self.b {
                return None;
            }
            self.y += 1;
            self.enter_row();
        }
    }
}

#[cfg(feature = "fill")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "ellipse-aa", feature = "fill"))))]
impl Fill<Plot> for EllipseAa {
    /// Filled anti-aliased ellipse: solid interior [`Span`]s plus edge points.
    ///
    /// For the implicit ellipse error
    /// `f = b²x² + a²y² − a²b²`, the half-pixel edge band is approximated by
    /// `g = sqrt(b⁴x² + a⁴y²)`, half the local gradient magnitude. Pixels with
    /// `f <= -g` are solid, pixels with `f >= g` are off, and the band between
    /// them is interpolated linearly. This is the ellipse counterpart of
    /// Vadillo's squared-distance circle fill and uses integer arithmetic only.
    /// The resulting intensity is a first-order coverage approximation, not an
    /// exact box-filtered pixel-area integral. Non-circular rows calculate one
    /// quadrant and reflect edge points and spans across both axes.
    fn fill(self) -> impl Iterator<Item = Plot> {
        EllipseAaFill::new(self.center, self.a, self.b)
    }
}

#[cfg(feature = "fill")]
enum FillPhase {
    Degenerate,
    Left,
    Solid,
    Center,
    Right,
    Done,
}

#[cfg(feature = "fill")]
struct EllipseAaFill {
    center: Point,
    a: isize,
    b: isize,
    a2: i128,
    b2: i128,
    target: i128,
    dy: isize,
    solid: isize,
    edge: isize,
    dx: isize,
    phase: FillPhase,
    reflect_rows: bool,
    pending: ArrayDeque<Plot, 4>,
}

#[cfg(feature = "fill")]
impl EllipseAaFill {
    fn new(center: Point, a: isize, b: isize) -> Self {
        let a2 = (a as i128) * (a as i128);
        let b2 = (b as i128) * (b as i128);
        let reflect_rows = a != b && a > 0 && b > 0;
        let mut fill = Self {
            center,
            a,
            b,
            a2,
            b2,
            target: a2 * b2,
            dy: if reflect_rows { 0 } else { -b },
            solid: -1,
            edge: 0,
            dx: 0,
            phase: if a == 0 || b == 0 {
                FillPhase::Degenerate
            } else {
                FillPhase::Left
            },
            reflect_rows,
            pending: ArrayDeque::new(),
        };
        if a > 0 && b > 0 {
            fill.enter_row();
        }
        fill
    }

    fn alpha(&self, dx: isize) -> u8 {
        if self.a == self.b {
            let r = self.a as i128;
            let d2 = (dx as i128) * (dx as i128) + (self.dy as i128) * (self.dy as i128);
            let inner = r * r - r;
            let outer = r * r + r;
            if d2 < inner {
                255
            } else if d2 < outer {
                (255 * (outer - d2) / (2 * r)) as u8
            } else {
                0
            }
        } else {
            let x = dx as i128;
            let y = self.dy as i128;
            let f = self.b2 * x * x + self.a2 * y * y - self.target;
            let gradient2 = self.b2 * self.b2 * x * x + self.a2 * self.a2 * y * y;
            let gradient = isqrt(gradient2 as u128) as i128;
            if gradient == 0 || f <= -gradient {
                255
            } else if f < gradient {
                (255 * (gradient - f) / (2 * gradient)) as u8
            } else {
                0
            }
        }
    }

    fn bound(&self, from: isize, solid: bool) -> isize {
        let accepts = |x| {
            if solid {
                if self.a == self.b {
                    let r = self.a as i128;
                    let x = x as i128;
                    let y = self.dy as i128;
                    x * x + y * y < r * r - r
                } else {
                    self.alpha(x) == 255
                }
            } else {
                self.alpha(x) > 0
            }
        };
        let mut x = from.max(0);
        while x < self.a && accepts(x + 1) {
            x += 1;
        }
        while x >= 0 && !accepts(x) {
            x -= 1;
        }
        x
    }

    fn enter_row(&mut self) {
        self.solid = self.bound(self.solid, true);
        self.edge = self.bound(self.edge, false);
        self.dx = self.edge;
        self.phase = FillPhase::Left;
    }

    fn push_pending(&mut self, plot: Plot) {
        if !self.pending.iter().any(|pending| *pending == plot) {
            self.pending
                .push_back(plot)
                .expect("EllipseAa fill pending overflow");
        }
    }

    fn prepare_edge_reflections(&mut self, dx: isize, alpha: u8) {
        for point in [
            (self.center.0 - dx, self.center.1 + self.dy),
            (self.center.0 + dx, self.center.1 + self.dy),
            (self.center.0 - dx, self.center.1 - self.dy),
            (self.center.0 + dx, self.center.1 - self.dy),
        ] {
            self.push_pending(Plot::Point((point, alpha)));
        }
    }

    fn prepare_span_reflections(&mut self) {
        for y in [self.center.1 + self.dy, self.center.1 - self.dy] {
            self.push_pending(Plot::Span(Span {
                x0: self.center.0 - self.solid,
                x1: self.center.0 + self.solid,
                y,
            }));
        }
    }

    fn next_reflected(&mut self) -> Option<Plot> {
        loop {
            if let Some(plot) = self.pending.pop_front() {
                return Some(plot);
            }
            match self.phase {
                FillPhase::Left => {
                    if self.dx > self.solid {
                        let dx = self.dx;
                        self.dx -= 1;
                        let alpha = self.alpha(dx);
                        if alpha > 0 {
                            self.prepare_edge_reflections(dx, alpha);
                        }
                    } else {
                        self.phase = FillPhase::Solid;
                    }
                }
                FillPhase::Solid => {
                    self.phase = FillPhase::Right;
                    if self.solid >= 0 {
                        self.prepare_span_reflections();
                    }
                }
                FillPhase::Right => {
                    if self.dy == self.b {
                        self.phase = FillPhase::Done;
                    } else {
                        self.dy += 1;
                        self.enter_row();
                    }
                }
                FillPhase::Done => return None,
                FillPhase::Degenerate | FillPhase::Center => unreachable!(),
            }
        }
    }
}

#[cfg(feature = "fill")]
impl Iterator for EllipseAaFill {
    type Item = Plot;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reflect_rows {
            return self.next_reflected();
        }
        loop {
            match self.phase {
                FillPhase::Degenerate => {
                    self.phase = FillPhase::Done;
                    if self.a == 0 && self.b == 0 {
                        return Some(Plot::Point((self.center, 255)));
                    }
                    return Some(Plot::Span(if self.b == 0 {
                        Span {
                            x0: self.center.0 - self.a,
                            x1: self.center.0 + self.a,
                            y: self.center.1,
                        }
                    } else {
                        let y = self.center.1 - self.b;
                        self.dy = -self.b + 1;
                        self.phase = FillPhase::Center;
                        Span {
                            x0: self.center.0,
                            x1: self.center.0,
                            y,
                        }
                    }));
                }
                FillPhase::Left => {
                    if self.dx > self.solid.max(0) {
                        let dx = self.dx;
                        self.dx -= 1;
                        let alpha = self.alpha(dx);
                        if alpha > 0 {
                            return Some(Plot::Point((
                                (self.center.0 - dx, self.center.1 + self.dy),
                                alpha,
                            )));
                        }
                    } else {
                        self.phase = FillPhase::Solid;
                    }
                }
                FillPhase::Solid => {
                    self.phase = FillPhase::Center;
                    if self.solid >= 0 {
                        return Some(Plot::Span(Span {
                            x0: self.center.0 - self.solid,
                            x1: self.center.0 + self.solid,
                            y: self.center.1 + self.dy,
                        }));
                    }
                }
                FillPhase::Center => {
                    if self.a == 0 && self.b > 0 {
                        if self.dy <= self.b {
                            let y = self.center.1 + self.dy;
                            self.dy += 1;
                            return Some(Plot::Span(Span {
                                x0: self.center.0,
                                x1: self.center.0,
                                y,
                            }));
                        }
                        self.phase = FillPhase::Done;
                        continue;
                    }
                    self.phase = FillPhase::Right;
                    self.dx = self.solid.max(0) + 1;
                    if self.solid < 0 {
                        let alpha = self.alpha(0);
                        if alpha > 0 {
                            return Some(Plot::Point((
                                (self.center.0, self.center.1 + self.dy),
                                alpha,
                            )));
                        }
                    }
                }
                FillPhase::Right => {
                    if self.dx <= self.edge {
                        let dx = self.dx;
                        self.dx += 1;
                        let alpha = self.alpha(dx);
                        if alpha > 0 {
                            return Some(Plot::Point((
                                (self.center.0 + dx, self.center.1 + self.dy),
                                alpha,
                            )));
                        }
                    } else if self.dy == self.b {
                        self.phase = FillPhase::Done;
                    } else {
                        self.dy += 1;
                        self.enter_row();
                    }
                }
                FillPhase::Done => return None,
            }
        }
    }
}

fn isqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let mut bit = 1u128 << 126;
    while bit > value {
        bit >>= 2;
    }
    let mut value = value;
    let mut root = 0u128;
    while bit != 0 {
        if value >= root + bit {
            value -= root + bit;
            root = (root >> 1) + bit;
        } else {
            root >>= 1;
        }
        bit >>= 2;
    }
    root
}

#[cfg(test)]
mod tests {
    use super::EllipseAa;
    use crate::CircleAa;
    #[cfg(feature = "fill")]
    use crate::{Fill, Plot};
    use std::collections::BTreeMap;
    use std::vec::Vec;

    fn blend(iter: impl Iterator<Item = crate::PointAa>) -> BTreeMap<crate::Point, u8> {
        let mut pixels: BTreeMap<crate::Point, u8> = BTreeMap::new();
        for (point, alpha) in iter {
            pixels
                .entry(point)
                .and_modify(|old| *old = (*old).max(alpha))
                .or_insert(alpha);
        }
        pixels
    }

    #[test]
    fn equal_radii_match_circle_aa_exactly() {
        for radius in 0..=32 {
            let ellipse: Vec<_> = EllipseAa::new((3, -2), radius, radius).collect();
            let circle: Vec<_> = CircleAa::new((3, -2), radius).collect();
            assert_eq!(ellipse, circle, "radius={radius}");
        }
    }

    #[test]
    fn degenerate_axes_and_negative_radii() {
        assert_eq!(
            EllipseAa::new((3, -2), 0, 0).collect::<Vec<_>>(),
            [((3, -2), 255)]
        );
        assert_eq!(
            EllipseAa::new((0, 0), 2, 0).collect::<Vec<_>>(),
            [
                ((-2, 0), 255),
                ((-1, 0), 255),
                ((0, 0), 255),
                ((1, 0), 255),
                ((2, 0), 255),
            ]
        );
        assert_eq!(
            EllipseAa::new((0, 0), 0, 2).collect::<Vec<_>>(),
            [
                ((0, -2), 255),
                ((0, -1), 255),
                ((0, 0), 255),
                ((0, 1), 255),
                ((0, 2), 255),
            ]
        );

        for &(a, b) in &[(3, 7), (7, 3), (0, 4), (4, 0)] {
            let positive: Vec<_> = EllipseAa::new((1, -1), a, b).collect();
            let negative: Vec<_> = EllipseAa::new((1, -1), -a, -b).collect();
            assert_eq!(positive, negative, "a={a} b={b}");
        }
    }

    #[test]
    fn outline_is_symmetric() {
        for &(a, b) in &[(1, 4), (5, 2), (3, 7), (12, 3)] {
            let pixels = blend(EllipseAa::new((0, 0), a, b));
            for (&(x, y), &alpha) in &pixels {
                assert_eq!(pixels.get(&(-x, y)), Some(&alpha), "a={a} b={b}");
                assert_eq!(pixels.get(&(x, -y)), Some(&alpha), "a={a} b={b}");
            }
        }
    }

    #[test]
    fn outline_never_yields_zero_coverage() {
        for a in 1..=16 {
            for b in 1..=16 {
                for (point, alpha) in EllipseAa::new((0, 0), a, b) {
                    assert!(alpha > 0, "a={a} b={b} point={point:?}");
                }
            }
        }
    }

    #[test]
    fn outline_transposes_with_radii() {
        for &(a, b) in &[(1, 64), (3, 40), (5, 12), (12, 5)] {
            let transposed: BTreeMap<_, _> = blend(EllipseAa::new((0, 0), a, b))
                .into_iter()
                .map(|((x, y), alpha)| ((y, x), alpha))
                .collect();
            assert_eq!(transposed, blend(EllipseAa::new((0, 0), b, a)));
        }
    }

    #[test]
    fn outline_shape() {
        let pixels = blend(EllipseAa::new((3, 3), 3, 2));
        let mut grid = [0u32; 8];
        for ((x, y), alpha) in pixels {
            let shift = ((7 - x) * 4) as u32;
            grid[y as usize] |= ((alpha as u32) >> 4) << shift;
        }
        assert_eq!(
            grid,
            [
                0x00000000, 0x07efe700, 0x98101890, 0xf00000f0, 0x98101890, 0x07efe700, 0x00000000,
                0x00000000,
            ]
        );
    }

    #[cfg(feature = "fill")]
    fn fill_pixels(a: isize, b: isize) -> BTreeMap<crate::Point, u8> {
        let mut pixels = BTreeMap::new();
        for plot in EllipseAa::new((0, 0), a, b).fill() {
            match plot {
                Plot::Span(span) => {
                    for x in span.x0..=span.x1 {
                        assert!(pixels.insert((x, span.y), 255).is_none());
                    }
                }
                Plot::Point((point, alpha)) => {
                    assert!(alpha > 0);
                    assert!(pixels.insert(point, alpha).is_none());
                }
            }
        }
        pixels
    }

    #[cfg(feature = "fill")]
    #[test]
    fn equal_radii_fills_match_circle_aa() {
        for radius in 0..=32 {
            let ellipse: Vec<_> = EllipseAa::new((0, 0), radius, radius).fill().collect();
            let circle: Vec<_> = CircleAa::new((0, 0), radius).fill().collect();
            assert_eq!(ellipse, circle, "radius={radius}");
        }
    }

    #[cfg(feature = "fill")]
    #[test]
    fn degenerate_and_negative_fills() {
        assert_eq!(
            fill_pixels(0, 2),
            [
                ((0, -2), 255),
                ((0, -1), 255),
                ((0, 0), 255),
                ((0, 1), 255),
                ((0, 2), 255),
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            fill_pixels(2, 0),
            [
                ((-2, 0), 255),
                ((-1, 0), 255),
                ((0, 0), 255),
                ((1, 0), 255),
                ((2, 0), 255),
            ]
            .into_iter()
            .collect()
        );

        for &(a, b) in &[(3, 7), (7, 3), (0, 4), (4, 0)] {
            let positive: Vec<_> = EllipseAa::new((1, -1), a, b).fill().collect();
            let negative: Vec<_> = EllipseAa::new((1, -1), -a, -b).fill().collect();
            assert_eq!(positive, negative, "a={a} b={b}");
        }
    }

    #[cfg(feature = "fill")]
    #[test]
    fn fill_is_symmetric() {
        for &(a, b) in &[(1, 4), (5, 2), (3, 7), (12, 3), (1, 64), (64, 1)] {
            let pixels = fill_pixels(a, b);
            for (&(x, y), &alpha) in &pixels {
                assert_eq!(pixels.get(&(-x, y)), Some(&alpha), "a={a} b={b}");
                assert_eq!(pixels.get(&(x, -y)), Some(&alpha), "a={a} b={b}");
            }
        }
    }

    #[cfg(feature = "fill")]
    #[test]
    fn fills_transpose_with_radii() {
        for &(a, b) in &[(1, 64), (3, 40), (5, 12), (12, 5)] {
            let transposed: BTreeMap<_, _> = fill_pixels(a, b)
                .into_iter()
                .map(|((x, y), alpha)| ((y, x), alpha))
                .collect();
            assert_eq!(transposed, fill_pixels(b, a), "a={a} b={b}");
        }
    }

    #[cfg(feature = "fill")]
    #[test]
    fn fill_shape() {
        let pixels = fill_pixels(3, 2);
        let mut grid = [0u32; 8];
        for ((x, y), alpha) in pixels {
            let shift = ((7 - (x + 3)) * 4) as u32;
            grid[(y + 3) as usize] |= ((alpha as u32) >> 4) << shift;
        }
        assert_eq!(
            grid,
            [
                0x00000000, 0x01676100, 0x3fffff30, 0x7fffff70, 0x3fffff30, 0x01676100, 0x00000000,
                0x00000000,
            ]
        );
    }
}
