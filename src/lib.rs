//! Iterator-based Bresenham rasterizers
//!
//! [Bresenham's line drawing algorithm]
//! (https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm) is a fast
//! integer algorithm to draw a line between two points. This crate implements
//! that line walker plus the other primitives from
//! [Alois Zingl's notes](https://zingl.github.io/bresenham.html) — 3D lines,
//! circles, ellipses, quadratic Bézier curves, and anti-aliased variants —
//! as iterators. It calculates coordinates without knowing anything about
//! drawing methods or surfaces.
//!
//! Example:
//!
//! ```rust
//! extern crate bresenham;
//! use bresenham::Bresenham;
//!
//! fn main() {
//!     for (x, y) in Bresenham::new((0, 1), (6, 4)) {
//!         println!("{}, {}", x, y);
//!     }
//! }
//! ```
//!
//! Will print:
//!
//! ```text
//! (0, 1)
//! (1, 1)
//! (2, 2)
//! (3, 2)
//! (4, 3)
//! (5, 3)
//! ```

#![no_std]

#[cfg(test)]
extern crate std;

use core::iter::Iterator;

mod aa;
mod bezier;
mod circle;
mod ellipse;
mod line3d;
mod plot;

pub use aa::{AaPixel, BresenhamAA, LineWidth, QuadBezierAA};
pub use bezier::QuadBezier;
pub use circle::Circle;
pub use ellipse::{Ellipse, EllipseRect};
pub use line3d::Bresenham3d;

/// Convenient typedef for two machine-sized integers
pub type Point = (isize, isize);

/// Convenient typedef for three machine-sized integers
pub type Point3 = (isize, isize, isize);

/// Line-drawing iterator
pub struct Bresenham {
    x: isize,
    y: isize,
    dx: isize,
    dy: isize,
    x1: isize,
    diff: isize,
    octant: Octant,
}

struct Octant(u8);

impl Octant {
    /// adapted from http://codereview.stackexchange.com/a/95551
    #[inline]
    fn from_points(start: Point, end: Point) -> Octant {
        let mut dx = end.0 - start.0;
        let mut dy = end.1 - start.1;

        let mut octant = 0;

        if dy < 0 {
            dx = -dx;
            dy = -dy;
            octant += 4;
        }

        if dx < 0 {
            let tmp = dx;
            dx = dy;
            dy = -tmp;
            octant += 2
        }

        if dx < dy {
            octant += 1
        }

        Octant(octant)
    }

    #[inline]
    fn to_octant0(&self, p: Point) -> Point {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (p.1, -p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (-p.1, p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn from_octant0(&self, p: Point) -> Point {
        match self.0 {
            0 => (p.0, p.1),
            1 => (p.1, p.0),
            2 => (-p.1, p.0),
            3 => (-p.0, p.1),
            4 => (-p.0, -p.1),
            5 => (-p.1, -p.0),
            6 => (p.1, -p.0),
            7 => (p.0, -p.1),
            _ => unreachable!(),
        }
    }

    /// Whether an exact error tie should step the minor axis.
    ///
    /// Classic Bresenham always steps on ties (`diff >= 0`). That bias is local
    /// to the octant, so reversing the endpoints (octant `n` vs `n + 4`) picks
    /// the opposite pixel. Stepping on ties only in octants 4–7 makes both
    /// directions choose the same world-space points.
    #[inline]
    fn step_minor_on_tie(&self) -> bool {
        self.0 >= 4
    }
}

impl Bresenham {
    /// Creates a new iterator. Yields points from `start` through `end`,
    /// inclusive. The set of points does not depend on direction:
    /// `Bresenham::new(a, b)` and `Bresenham::new(b, a)` visit the same
    /// pixels (in reverse order).
    #[inline]
    pub fn new(start: Point, end: Point) -> Bresenham {
        let octant = Octant::from_points(start, end);

        let start = octant.to_octant0(start);
        let end = octant.to_octant0(end);

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        Bresenham {
            x: start.0,
            y: start.1,
            dx: dx,
            dy: dy,
            x1: end.0,
            diff: 2 * dy - dx,
            octant: octant,
        }
    }
}

impl Iterator for Bresenham {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // "The endpoints of the [bresenham] line are the pixels at (x0, y0) and
        // (x1, y1) where the first coordinate of the pair is the column and the
        // second is the row."
        if self.x > self.x1 {
            return None;
        }

        let p = (self.x, self.y);

        if self.diff > 0 || (self.diff == 0 && self.octant.step_minor_on_tie()) {
            self.y += 1;
            self.diff -= 2 * self.dx;
        }

        self.diff += 2 * self.dy;

        // loop inc
        self.x += 1;

        Some(self.octant.from_octant0(p))
    }
}

#[cfg(test)]
mod tests {
    use super::Bresenham;
    use std::vec::Vec;

    #[test]
    fn test_wp_example() {
        let bi = Bresenham::new((0, 1), (6, 4));
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(0, 1), (1, 2), (2, 2), (3, 3), (4, 3), (5, 4), (6, 4)]
        )
    }

    #[test]
    fn test_inverse_wp() {
        let bi = Bresenham::new((6, 4), (0, 1));
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(6, 4), (5, 4), (4, 3), (3, 3), (2, 2), (1, 2), (0, 1)]
        )
    }

    #[test]
    fn test_direction_symmetric() {
        for x0 in -8..=8 {
            for y0 in -8..=8 {
                for x1 in -8..=8 {
                    for y1 in -8..=8 {
                        let fwd: Vec<_> = Bresenham::new((x0, y0), (x1, y1)).collect();
                        let rev: Vec<_> = Bresenham::new((x1, y1), (x0, y0)).collect();
                        assert!(
                            fwd.iter().rev().eq(rev.iter()),
                            "asymmetric line ({}, {}) -> ({}, {}): {:?} vs {:?}",
                            x0,
                            y0,
                            x1,
                            y1,
                            fwd,
                            rev
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_straight_hline() {
        let bi = Bresenham::new((2, 3), (5, 3));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (3, 3), (4, 3), (5, 3)]);
    }

    #[test]
    fn test_straight_vline() {
        let bi = Bresenham::new((2, 3), (2, 6));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (2, 4), (2, 5), (2, 6)]);
    }

    #[test]
    fn test_line3d() {
        let res: Vec<_> = super::Bresenham3d::new((0, 0, 0), (2, 1, 0)).collect();
        assert_eq!(res, [(0, 0, 0), (1, 0, 0), (2, 1, 0)]);

        let res: Vec<_> = super::Bresenham3d::new((0, 0, 0), (3, 3, 3)).collect();
        assert_eq!(res, [(0, 0, 0), (1, 1, 1), (2, 2, 2), (3, 3, 3)]);

        let res: Vec<_> = super::Bresenham3d::new((1, 2, 3), (1, 2, 3)).collect();
        assert_eq!(res, [(1, 2, 3)]);
    }

    #[test]
    fn test_circle() {
        let res: Vec<_> = super::Circle::new((0, 0), 0).collect();
        assert_eq!(res, [(0, 0)]);

        let res: Vec<_> = super::Circle::new((0, 0), 1).collect();
        assert_eq!(res, [(1, 0), (0, 1), (-1, 0), (0, -1)]);

        let res: Vec<_> = super::Circle::new((5, 5), 2).collect();
        assert_eq!(
            res,
            [
                (7, 5),
                (5, 7),
                (3, 5),
                (5, 3),
                (7, 6),
                (4, 7),
                (3, 4),
                (6, 3),
                (6, 7),
                (3, 6),
                (4, 3),
                (7, 4)
            ]
        );

        let res: Vec<_> = super::Circle::new((0, 0), 4).collect();
        assert_eq!(
            res,
            [
                (4, 0),
                (0, 4),
                (-4, 0),
                (0, -4),
                (4, 1),
                (-1, 4),
                (-4, -1),
                (1, -4),
                (3, 2),
                (-2, 3),
                (-3, -2),
                (2, -3),
                (2, 3),
                (-3, 2),
                (-2, -3),
                (3, -2),
                (1, 4),
                (-4, 1),
                (-1, -4),
                (4, -1)
            ]
        );
    }

    #[test]
    fn test_ellipse() {
        let res: Vec<_> = super::Ellipse::new((0, 0), 5, 2).collect();
        assert_eq!(
            res,
            [
                (5, 0),
                (-5, 0),
                (-5, 0),
                (5, 0),
                (4, 1),
                (-4, 1),
                (-4, -1),
                (4, -1),
                (3, 2),
                (-3, 2),
                (-3, -2),
                (3, -2),
                (2, 2),
                (-2, 2),
                (-2, -2),
                (2, -2),
                (1, 2),
                (-1, 2),
                (-1, -2),
                (1, -2),
                (0, 2),
                (0, 2),
                (0, -2),
                (0, -2)
            ]
        );

        let res: Vec<_> = super::Ellipse::new((0, 0), 1, 4).collect();
        assert_eq!(
            res,
            [
                (1, 0),
                (-1, 0),
                (-1, 0),
                (1, 0),
                (1, 1),
                (-1, 1),
                (-1, -1),
                (1, -1),
                (1, 2),
                (-1, 2),
                (-1, -2),
                (1, -2),
                (0, 3),
                (0, 3),
                (0, -3),
                (0, -3),
                (0, 4),
                (0, -4)
            ]
        );
    }

    #[test]
    fn test_ellipse_rect() {
        let res: Vec<_> = super::EllipseRect::new((0, 0), (8, 4)).collect();
        assert_eq!(
            res,
            [
                (8, 2),
                (0, 2),
                (0, 2),
                (8, 2),
                (7, 3),
                (1, 3),
                (1, 1),
                (7, 1),
                (6, 4),
                (2, 4),
                (2, 0),
                (6, 0),
                (5, 4),
                (3, 4),
                (3, 0),
                (5, 0),
                (4, 4),
                (4, 4),
                (4, 0),
                (4, 0),
                (4, 4),
                (4, 4),
                (4, 0),
                (4, 0)
            ]
        );
    }

    #[test]
    fn test_circle_for_each_matches_iter() {
        for r in 0..16 {
            let a: Vec<_> = super::Circle::new((3, -2), r).collect();
            let mut b = Vec::new();
            super::Circle::new((3, -2), r).for_each(|p| b.push(p));
            assert_eq!(a, b, "r={r}");
        }
    }

    #[test]
    fn test_ellipse_rect_for_each_matches_iter() {
        for &(p0, p1) in &[
            ((0, 0), (8, 4)),
            ((0, 0), (0, 0)),
            ((2, 3), (12, 10)),
            ((10, 1), (1, 8)),
        ] {
            let a: Vec<_> = super::EllipseRect::new(p0, p1).collect();
            let mut b = Vec::new();
            super::EllipseRect::new(p0, p1).for_each(|p| b.push(p));
            assert_eq!(a, b, "{p0:?} {p1:?}");
        }
    }

    #[test]
    fn test_quad_bezier() {
        let res: Vec<_> = super::QuadBezier::new((0, 0), (2, 0), (4, 0)).collect();
        assert_eq!(res, [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]);

        let res: Vec<_> = super::QuadBezier::new((0, 0), (2, 4), (4, 0)).collect();
        assert_eq!(res, [(0, 0), (1, 1), (2, 2), (4, 0), (3, 1), (2, 2)]);

        let res: Vec<_> = super::QuadBezier::new((0, 0), (1, 3), (4, 3)).collect();
        assert_eq!(res, [(0, 0), (0, 1), (1, 2), (2, 3), (3, 3), (4, 3)]);
    }

    #[test]
    fn test_line_aa() {
        let res: Vec<_> = super::BresenhamAA::new((0, 0), (4, 0)).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 0),
                ((1, 0), 0),
                ((2, 0), 0),
                ((3, 0), 0),
                ((4, 0), 0)
            ]
        );

        let res: Vec<_> = super::BresenhamAA::new((0, 1), (6, 4)).collect();
        assert_eq!(
            res,
            [
                ((0, 1), 0),
                ((1, 1), 127),
                ((1, 2), 127),
                ((2, 2), 0),
                ((3, 2), 127),
                ((3, 3), 127),
                ((4, 3), 0),
                ((5, 3), 127),
                ((5, 4), 127),
                ((6, 4), 0)
            ]
        );

        let res: Vec<_> = super::BresenhamAA::new((0, 0), (3, 3)).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 0),
                ((0, 1), 191),
                ((1, 0), 191),
                ((1, 1), 0),
                ((1, 2), 191),
                ((2, 1), 191),
                ((2, 2), 0),
                ((2, 3), 191),
                ((3, 2), 191),
                ((3, 3), 0)
            ]
        );
    }

    #[test]
    fn test_line_width() {
        let res: Vec<_> = super::LineWidth::new((0, 0), (4, 0), 1.0).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 0),
                ((1, 0), 0),
                ((2, 0), 0),
                ((3, 0), 0),
                ((4, 0), 0)
            ]
        );

        let res: Vec<_> = super::LineWidth::new((0, 0), (5, 2), 3.0).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 0),
                ((0, 1), 0),
                ((0, 2), 218),
                ((1, 0), 0),
                ((1, 1), 0),
                ((1, 2), 123),
                ((2, 0), 0),
                ((3, 0), 29),
                ((4, 0), 123),
                ((5, 0), 218),
                ((2, 1), 0),
                ((2, 2), 29),
                ((3, 1), 0),
                ((3, 2), 0),
                ((3, 3), 171),
                ((4, 1), 0),
                ((4, 2), 0),
                ((4, 3), 76),
                ((5, 1), 0),
                ((5, 2), 0),
                ((5, 3), 0),
                ((5, 4), 218)
            ]
        );
    }

    #[test]
    fn test_quad_bezier_aa() {
        let res: Vec<_> = super::QuadBezierAA::new((0, 0), (1, 3), (4, 3)).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 0),
                ((1, 0), 236),
                ((0, 1), 102),
                ((0, 2), 225),
                ((1, 1), 120),
                ((1, 2), 37),
                ((1, 3), 233),
                ((2, 2), 126),
                ((2, 3), 103),
                ((3, 2), 237),
                ((3, 3), 22),
                ((3, 3), 0),
                ((4, 3), 0)
            ]
        );
    }
}
