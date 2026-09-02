//! Anti-aliased circle from Fu and Niu's integral algorithm.
//!
//! B. Fu and L. Niu, "Integral Algorithm for Generating Anti-Aliasing Circle
//! Based on Bresenham Algorithm", *Advanced Materials Research*,
//! 490–495:1202–1206, 2012.
//!
//! Coverage is the crate convention: `255` is fully on the curve, `0` is
//! fully off.

use arraydeque::ArrayDeque;

use crate::{Point, PointAa};

/// Anti-aliased axis-aligned circle
///
/// Integer-only. Walks the first octant with the Bresenham circle decision
/// and derives each pixel's coverage from the signed distance to the true
/// arc, `f / 2y`, where `f = x² + y² − r²` is maintained incrementally
/// (Fu & Niu, 2012). Each step yields the Bresenham pixel plus its radial
/// neighbor, mirrored eight ways.
///
/// Near the octant seams a pixel may be yielded more than once with
/// different coverage (also true of [`QuadBezierAa`](crate::QuadBezierAa));
/// blend with `max` when compositing.
pub struct CircleAa {
    xm: isize,
    ym: isize,
    x: isize,
    y: isize,
    /// `x² + y² − r²` for the current octant pixel `(x, y)`.
    f: isize,
    pending: ArrayDeque<PointAa, 16>,
    done: bool,
}

impl CircleAa {
    /// Closed anti-aliased circle centered at `center` with the given
    /// `radius`.
    ///
    /// A radius of `0` yields the center point once. Negative radii are
    /// treated as their absolute value.
    pub fn new(center: Point, radius: isize) -> Self {
        let r = radius.abs();
        let mut c = CircleAa {
            xm: center.0,
            ym: center.1,
            x: 0,
            y: r,
            f: 0,
            pending: ArrayDeque::new(),
            done: false,
        };
        if r == 0 {
            c.push(center, 255);
            c.done = true;
        }
        c
    }

    fn push(&mut self, p: Point, coverage: u8) {
        self.pending
            .push_back((p, coverage))
            .expect("CircleAa pending overflow");
    }

    /// Push the eight symmetric copies of octant pixel `(x, y)`, skipping the
    /// duplicates that arise on the axes (`x == 0`, `y == 0`) and the
    /// diagonal (`x == y`).
    fn push8(&mut self, x: isize, y: isize, coverage: u8) {
        let (xm, ym) = (self.xm, self.ym);
        let candidates = [
            (xm + x, ym + y),
            (xm - x, ym + y),
            (xm + x, ym - y),
            (xm - x, ym - y),
            (xm + y, ym + x),
            (xm - y, ym + x),
            (xm + y, ym - x),
            (xm - y, ym - x),
        ];
        let start = self.pending.len();
        'candidate: for p in candidates {
            for prior in self.pending.iter().skip(start) {
                if prior.0 == p {
                    continue 'candidate;
                }
            }
            self.push(p, coverage);
        }
    }

    fn step(&mut self) {
        // Signed vertical distance from pixel (x, y) to the arc is f / 2y up
        // to a dropped ε²/2y term (Fu & Niu eq. 3; their numerators
        // D ± (2y − 1) equal 2f).
        let fade = (255 * self.f.abs() / (2 * self.y)).min(255) as u8;
        self.push8(self.x, self.y, 255 - fade);
        if fade > 0 {
            // f > 0: pixel is outside the arc, so the arc continues toward
            // the center (y − 1); f < 0: inside, toward y + 1. The neighbor
            // receives the remaining coverage.
            let ny = if self.f > 0 { self.y - 1 } else { self.y + 1 };
            self.push8(self.x, ny, fade);
        }

        // Bresenham advance. The paper's decision parameter for column x + 1
        // is D = 2·f(x+1, y) − 2y + 1; D ≥ 0 steps down to y − 1.
        self.x += 1;
        self.f += 2 * self.x - 1;
        if 2 * self.f - 2 * self.y + 1 >= 0 {
            self.f -= 2 * self.y - 1;
            self.y -= 1;
        }
        if self.x > self.y {
            self.done = true;
        }
    }
}

impl Iterator for CircleAa {
    type Item = PointAa;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(p) = self.pending.pop_front() {
                return Some(p);
            }
            if self.done {
                return None;
            }
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CircleAa;
    use std::vec::Vec;

    #[test]
    fn test_circle_aa_degenerate() {
        let res: Vec<_> = CircleAa::new((3, -2), 0).collect();
        assert_eq!(res, [((3, -2), 255)]);

        let res: Vec<_> = CircleAa::new((0, 0), 1).collect();
        assert_eq!(
            res,
            [((0, 1), 255), ((0, -1), 255), ((1, 0), 255), ((-1, 0), 255)]
        );
    }

    #[test]
    fn test_circle_aa_negative_radius() {
        let pos: Vec<_> = CircleAa::new((1, 2), 5).collect();
        let neg: Vec<_> = CircleAa::new((1, 2), -5).collect();
        assert_eq!(pos, neg);
    }

    #[test]
    fn test_circle_aa_r2() {
        let res: Vec<_> = CircleAa::new((0, 0), 2).collect();
        assert_eq!(
            res,
            [
                ((0, 2), 255),
                ((0, -2), 255),
                ((2, 0), 255),
                ((-2, 0), 255),
                ((1, 2), 192),
                ((-1, 2), 192),
                ((1, -2), 192),
                ((-1, -2), 192),
                ((2, 1), 192),
                ((-2, 1), 192),
                ((2, -1), 192),
                ((-2, -1), 192),
                ((1, 1), 63),
                ((-1, 1), 63),
                ((1, -1), 63),
                ((-1, -1), 63)
            ]
        );
    }

    /// After max-blending duplicates, every pixel's coverage must be close to
    /// `255·(1 − d)` where `d` is the distance to the true arc along the
    /// pixel's minor axis.
    #[test]
    fn test_circle_aa_accuracy() {
        use std::collections::BTreeMap;

        for r in 2isize..=64 {
            let mut blended: BTreeMap<(isize, isize), u8> = BTreeMap::new();
            for (p, coverage) in CircleAa::new((0, 0), r) {
                let entry = blended.entry(p).or_insert(0);
                *entry = (*entry).max(coverage);
            }
            for (&(px, py), &coverage) in &blended {
                let (fx, fy) = (px as f64, py as f64);
                let rf = r as f64;
                let dist = if fy.abs() >= fx.abs() {
                    (fy.abs() - (rf * rf - fx * fx).sqrt()).abs()
                } else {
                    (fx.abs() - (rf * rf - fy * fy).sqrt()).abs()
                };
                let expected = 255.0 * (1.0 - dist).max(0.0);
                let err = (coverage as f64 - expected).abs();
                // The dropped ε²/2y term dominates at small radii.
                let tol = if r < 8 { 40.0 } else { 16.0 };
                assert!(
                    err <= tol,
                    "r={r} p=({px},{py}) coverage={coverage} expected={expected:.1}"
                );
            }
        }
    }

    /// The point set is symmetric under reflection across both axes and the
    /// diagonal.
    #[test]
    fn test_circle_aa_symmetric() {
        use std::collections::BTreeSet;

        for r in 0isize..=32 {
            let pts: BTreeSet<_> = CircleAa::new((0, 0), r).collect();
            for &((x, y), c) in &pts {
                assert!(pts.contains(&((-x, y), c)), "r={r} mirror x of ({x},{y})");
                assert!(pts.contains(&((x, -y), c)), "r={r} mirror y of ({x},{y})");
                assert!(pts.contains(&((y, x), c)), "r={r} swap of ({x},{y})");
            }
        }
    }

    /// Away from the octant seams, each column holds the Bresenham pixel and
    /// its neighbor, and they split full coverage between them.
    #[test]
    fn test_circle_aa_coverage_splits() {
        for r in 4isize..=32 {
            let pts: Vec<_> = CircleAa::new((0, 0), r).collect();
            // Strict first-octant interior: seam mirrors land at py <= px + 1,
            // so columns whose pixels all sit at py >= px + 3 are clean.
            for &((x, y), c) in &pts {
                if !(x > 0 && y >= x + 3 && c >= 128) {
                    continue;
                }
                // `(x, y)` is a main pixel. Its complement, if any, is the
                // vertical neighbor.
                if c == 255 {
                    continue;
                }
                let partner: isize = pts
                    .iter()
                    .filter(|&&((px, py), _)| px == x && (py - y).abs() == 1)
                    .map(|&(_, pc)| pc as isize)
                    .sum();
                assert_eq!(
                    c as isize + partner,
                    255,
                    "r={r} main ({x},{y}) coverage {c} partner {partner}"
                );
            }
        }
    }
}
