//! Anti-aliased primitives from Alois Zingl's `plotLineAA` and `plotQuadBezierSegAA`.
//!
//! Coverage is inverted from Zingl's `setPixelAA`: `255` is fully on the curve,
//! `0` is fully off.

use crate::{Point, PointAa};

fn coverage_i(zingl_fade: isize) -> u8 {
    255 - if zingl_fade < 0 {
        0
    } else if zingl_fade > 255 {
        255
    } else {
        zingl_fade as u8
    }
}

fn coverage_f(zingl_fade: f64) -> u8 {
    255 - if zingl_fade <= 0.0 {
        0
    } else if zingl_fade >= 255.0 {
        255
    } else {
        zingl_fade as u8
    }
}

/// Anti-aliased 2D line
///
/// Inclusive: `[start, end]`.
/// Source: Zingl `plotLineAA`
pub struct LineAa {
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    dx: isize,
    dy: isize,
    sx: isize,
    sy: isize,
    err: isize,
    ed: usize,
    pending: [PointAa; 3],
    pending_len: u8,
    pending_i: u8,
    done: bool,
}

/// An integer square root
/// 
/// Computes `floor(sqrt(x))` using a binary digit-by-digit square-root
/// algorithm.
///
/// This is the radix-2 specialization of the traditional digit-by-digit
/// ("longhand") square-root extraction method. Since binary root digits are
/// either 0 or 1, each digit can be selected using only a comparison and
/// subtraction. The radicand is processed in pairs of bits, hence the
/// descending powers of four.
///
/// M. Guy, "Fast Integer Square Root by Mr. Woo's Abacus Algorithm",
/// *University of Kent at Canterbury*, 1985.
///
/// The underlying digit-by-digit square-root method is much older and is
/// generally traced to work by François Viète around 1600.
fn isqrt(x: usize) -> usize {
    let mut n = x;
    let mut result: usize = 0;
    // Start with the largest representable power of 4.
    //
    // usize::BITS is a power of two on normal Rust targets, so BITS - 2 is
    // even and this gives 4^k rather than merely a power of two.
    let mut bit: usize = 1 << (usize::BITS - 2);
    // Find the largest power of 4 that does not exceed x. This identifies
    // the most significant pair of input bits that can contribute to sqrt(x).
    while bit > n {
        bit >>= 2;
    }
    // Determine one binary digit of the square root per iteration.
    while bit != 0 {
        // `result + bit` is the trial divisor corresponding to setting the
        // current root bit. If the remaining radicand can accommodate it,
        // accept that bit and subtract the trial value.
        if n >= result + bit {
            n -= result + bit;
            result = (result >> 1) + bit;
        } else {
            // The candidate bit does not fit, so this root bit is zero.
            result >>= 1;
        }

        // Move to the next pair of input bits / next binary root digit.
        bit >>= 2;
    }
    result
}

impl LineAa {
    /// Inclusive anti-aliased line (`[start, end]`).
    pub fn new(start: Point, end: Point) -> Self {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let ed = if dx + dy == 0 {
            1
        } else {
            isqrt((dx * dx + dy * dy) as usize)
        };
        let ed = if ed == 0 { 1 } else { ed };

        LineAa {
            x0,
            y0,
            x1,
            y1,
            dx,
            dy,
            sx,
            sy,
            err: dx - dy,
            ed,
            pending: [((0, 0), 0); 3],
            pending_len: 0,
            pending_i: 0,
            done: false,
        }
    }

    fn push(&mut self, p: Point, fade: u8) {
        self.pending[self.pending_len as usize] = (p, fade);
        self.pending_len += 1;
    }

    fn pop_pending(&mut self) -> Option<PointAa> {
        if self.pending_i < self.pending_len {
            let p = self.pending[self.pending_i as usize];
            self.pending_i += 1;
            if self.pending_i == self.pending_len {
                self.pending_i = 0;
                self.pending_len = 0;
            }
            Some(p)
        } else {
            None
        }
    }
}

impl Iterator for LineAa {
    type Item = PointAa;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.pop_pending() {
            return Some(p);
        }
        if self.done {
            return None;
        }

        let ed = self.ed as isize;
        let fade = coverage_i(255 * (self.err - self.dx + self.dy).abs() / ed);
        self.push((self.x0, self.y0), fade);

        let e2 = self.err;
        let x2 = self.x0;
        if 2 * e2 >= -self.dx {
            if self.x0 == self.x1 {
                self.done = true;
                return self.pop_pending();
            }
            if e2 + self.dy < ed {
                self.push(
                    (self.x0, self.y0 + self.sy),
                    coverage_i(255 * (e2 + self.dy) / ed),
                );
            }
            self.err -= self.dy;
            self.x0 += self.sx;
        }
        if 2 * e2 <= self.dy {
            if self.y0 == self.y1 {
                self.done = true;
                return self.pop_pending();
            }
            if self.dx - e2 < ed {
                self.push(
                    (x2 + self.sx, self.y0),
                    coverage_i(255 * (self.dx - e2) / ed),
                );
            }
            self.err += self.dx;
            self.y0 += self.sy;
        }

        self.pop_pending()
    }
}

enum BezierAaState {
    Curve,
    Line(LineAa),
    Done,
}

/// Anti-aliased quadratic Bézier
///
/// Like the C original, the gradient sign must not change along the segment;
/// if it does, the remainder is finished with an anti-aliased line.
///
/// Inclusive: `[p0, p2]`
/// Source: Zingl `plotQuadBezierSegAA`
pub struct QuadBezierAa {
    x0: isize,
    y0: isize,
    x2: isize,
    y2: isize,
    sx: isize,
    sy: isize,
    xx: i64,
    yy: i64,
    xy: i64,
    dx: f64,
    dy: f64,
    err: f64,
    state: BezierAaState,
    pending: [PointAa; 3],
    pending_len: u8,
    pending_i: u8,
}

impl QuadBezierAa {
    /// Inclusive quadratic Bézier (`[p0, p2]`) with control point `p1`.
    pub fn new(p0: Point, p1: Point, p2: Point) -> Self {
        let (mut x0, mut y0) = p0;
        let (x1, y1) = p1;
        let (mut x2, mut y2) = p2;
        let mut sx = x2 - x1;
        let mut sy = y2 - y1;
        let mut xx = (x0 - x1) as i64;
        let mut yy = (y0 - y1) as i64;
        let mut cur = (xx * sy as i64 - yy * sx as i64) as f64;

        if (sx as i64) * (sx as i64) + (sy as i64) * (sy as i64) > xx * xx + yy * yy {
            x2 = x0;
            x0 = sx + x1;
            y2 = y0;
            y0 = sy + y1;
            cur = -cur;
        }

        if cur != 0.0 {
            xx += sx as i64;
            sx = if x0 < x2 { 1 } else { -1 };
            xx *= sx as i64;
            yy += sy as i64;
            sy = if y0 < y2 { 1 } else { -1 };
            yy *= sy as i64;
            let mut xy = 2 * xx * yy;
            xx *= xx;
            yy *= yy;
            if cur * (sx as f64) * (sy as f64) < 0.0 {
                xx = -xx;
                yy = -yy;
                xy = -xy;
                cur = -cur;
            }
            let dx = 4.0 * (sy as f64) * ((x1 - x0) as f64) * cur + (xx - xy) as f64;
            let dy = 4.0 * (sx as f64) * ((y0 - y1) as f64) * cur + (yy - xy) as f64;
            xx += xx;
            yy += yy;
            let err = dx + dy + xy as f64;
            return QuadBezierAa {
                x0,
                y0,
                x2,
                y2,
                sx,
                sy,
                xx,
                yy,
                xy,
                dx,
                dy,
                err,
                state: BezierAaState::Curve,
                pending: [((0, 0), 0); 3],
                pending_len: 0,
                pending_i: 0,
            };
        }

        QuadBezierAa {
            x0,
            y0,
            x2,
            y2,
            sx: 0,
            sy: 0,
            xx: 0,
            yy: 0,
            xy: 0,
            dx: 0.0,
            dy: 0.0,
            err: 0.0,
            state: BezierAaState::Line(LineAa::new((x0, y0), (x2, y2))),
            pending: [((0, 0), 0); 3],
            pending_len: 0,
            pending_i: 0,
        }
    }

    fn push(&mut self, p: Point, fade: u8) {
        self.pending[self.pending_len as usize] = (p, fade);
        self.pending_len += 1;
    }

    fn pop_pending(&mut self) -> Option<PointAa> {
        if self.pending_i < self.pending_len {
            let p = self.pending[self.pending_i as usize];
            self.pending_i += 1;
            if self.pending_i == self.pending_len {
                self.pending_i = 0;
                self.pending_len = 0;
            }
            Some(p)
        } else {
            None
        }
    }

    fn step_curve(&mut self) {
        let cur = (self.dx + self.xy as f64).min(-self.xy as f64 - self.dy);
        let mut ed = (self.dx + self.xy as f64).max(-self.xy as f64 - self.dy);
        ed += 2.0 * ed * cur * cur / (4.0 * ed * ed + cur * cur);
        let fade = coverage_f(255.0 * (self.err - self.dx - self.dy - self.xy as f64).abs() / ed);
        self.push((self.x0, self.y0), fade);

        if self.x0 == self.x2 || self.y0 == self.y2 {
            self.state = BezierAaState::Line(LineAa::new((self.x0, self.y0), (self.x2, self.y2)));
            return;
        }

        let x1 = self.x0;
        let cur = self.dx - self.err;
        let step_y = 2.0 * self.err + self.dy < 0.0;
        if 2.0 * self.err + self.dx > 0.0 {
            if self.err - self.dy < ed {
                self.push(
                    (self.x0, self.y0 + self.sy),
                    coverage_f(255.0 * (self.err - self.dy).abs() / ed),
                );
            }
            self.x0 += self.sx;
            self.dx -= self.xy as f64;
            self.dy += self.yy as f64;
            self.err += self.dy;
        }
        if step_y {
            if cur < ed {
                self.push((x1 + self.sx, self.y0), coverage_f(255.0 * cur.abs() / ed));
            }
            self.y0 += self.sy;
            self.dy -= self.xy as f64;
            self.dx += self.xx as f64;
            self.err += self.dx;
        }

        if !(self.dy < self.dx) {
            self.state = BezierAaState::Line(LineAa::new((self.x0, self.y0), (self.x2, self.y2)));
        }
    }
}

impl Iterator for QuadBezierAa {
    type Item = PointAa;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(p) = self.pop_pending() {
                return Some(p);
            }
            match self.state {
                BezierAaState::Done => return None,
                BezierAaState::Line(ref mut line) => {
                    return line.next().or_else(|| {
                        self.state = BezierAaState::Done;
                        None
                    })
                }
                BezierAaState::Curve => self.step_curve(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{isqrt, LineAa, QuadBezierAa};
    use std::vec::Vec;

    fn f64_floor_sqrt(x: usize) -> usize {
        (x as f64).sqrt() as usize
    }

    /// `(x as f64).sqrt() as usize` matches `isqrt` only while `x` is an integer
    /// `f64` can represent (every integer through 2^53).
    fn f64_preserves(x: usize) -> bool {
        x as f64 as usize == x
    }

    #[test]
    fn isqrt_matches_libm_small_values() {
        for x in 0usize..=10_000 {
            assert_eq!(isqrt(x), f64_floor_sqrt(x), "x = {x}");
        }
    }

    #[test]
    fn isqrt_matches_libm_around_perfect_squares() {
        for n in 0usize..=1 << 16 {
            let Some(square) = n.checked_mul(n) else {
                break;
            };
            assert_eq!(isqrt(square), n, "sqrt({n}^2)");
            assert_eq!(isqrt(square), f64_floor_sqrt(square), "x = {square}");
            if square > 0 {
                assert_eq!(isqrt(square - 1), n - 1, "sqrt({n}^2 - 1)");
                assert_eq!(
                    isqrt(square - 1),
                    f64_floor_sqrt(square - 1),
                    "x = {square} - 1"
                );
            }
            if let Some(above) = square.checked_add(1) {
                if f64_preserves(above) {
                    let expected = if n == 0 { 1 } else { n };
                    assert_eq!(isqrt(above), expected, "sqrt({n}^2 + 1)");
                    assert_eq!(isqrt(above), f64_floor_sqrt(above), "x = {above}");
                }
            }
        }
    }

    #[test]
    fn isqrt_matches_libm_line_length_inputs() {
        // LineAa passes `dx * dx + dy * dy` to `isqrt`.
        for dx in 0isize..=256 {
            for dy in 0isize..=256 {
                let x = (dx * dx + dy * dy) as usize;
                assert_eq!(isqrt(x), f64_floor_sqrt(x), "dx = {dx}, dy = {dy}");
            }
        }
    }

    #[test]
    fn isqrt_matches_libm_where_f64_is_exact() {
        for shift in [20u32, 24, 31, 32, 40, 52, 53] {
            if shift >= usize::BITS {
                continue;
            }
            for x in [(1usize << shift) - 1, 1 << shift] {
                if !f64_preserves(x) {
                    continue;
                }
                assert_eq!(isqrt(x), f64_floor_sqrt(x), "x = {x}");
                if x > 0 {
                    assert_eq!(isqrt(x - 1), f64_floor_sqrt(x - 1), "x = {x} - 1");
                }
            }
        }
    }

    #[test]
    fn test_line_aa() {
        let res: Vec<_> = LineAa::new((0, 0), (4, 0)).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 255),
                ((1, 0), 255),
                ((2, 0), 255),
                ((3, 0), 255),
                ((4, 0), 255)
            ]
        );

        let res: Vec<_> = LineAa::new((0, 1), (6, 4)).collect();
        assert_eq!(
            res,
            [
                ((0, 1), 255),
                ((1, 1), 128),
                ((1, 2), 128),
                ((2, 2), 255),
                ((3, 2), 128),
                ((3, 3), 128),
                ((4, 3), 255),
                ((5, 3), 128),
                ((5, 4), 128),
                ((6, 4), 255)
            ]
        );

        let res: Vec<_> = LineAa::new((0, 0), (3, 3)).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 255),
                ((0, 1), 64),
                ((1, 0), 64),
                ((1, 1), 255),
                ((1, 2), 64),
                ((2, 1), 64),
                ((2, 2), 255),
                ((2, 3), 64),
                ((3, 2), 64),
                ((3, 3), 255)
            ]
        );
    }

    #[test]
    fn test_quad_bezier_aa() {
        let res: Vec<_> = QuadBezierAa::new((0, 0), (1, 3), (4, 3)).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 255),
                ((1, 0), 19),
                ((0, 1), 153),
                ((0, 2), 30),
                ((1, 1), 135),
                ((1, 2), 218),
                ((1, 3), 22),
                ((2, 2), 129),
                ((2, 3), 152),
                ((3, 2), 18),
                ((3, 3), 233),
                ((3, 3), 255),
                ((4, 3), 255)
            ]
        );
    }
}
