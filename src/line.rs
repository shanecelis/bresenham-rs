//! 2D Bresenham line (Cargo feature `line`).

use crate::Point;

/// Line-drawing iterator. Half-open: yields `start..end`.
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
    /// Creates a new iterator. Yields points from `start` toward `end`,
    /// excluding `end` (`start..end`). The set of points does not depend on
    /// direction: `Bresenham::new(a, b)` and `Bresenham::new(b, a)` visit the
    /// same interior pixels (in reverse order).
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
        if self.x >= self.x1 {
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
            [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3)]
        )
    }

    #[test]
    fn test_inverse_wp() {
        let bi = Bresenham::new((6, 4), (0, 1));
        let res: Vec<_> = bi.collect();

        assert_eq!(
            res,
            [(6, 4), (5, 3), (4, 3), (3, 2), (2, 2), (1, 1)]
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
                        let mut fwd_inc = fwd.clone();
                        let mut rev_inc = rev.clone();
                        if (x0, y0) != (x1, y1) {
                            fwd_inc.push((x1, y1));
                            rev_inc.push((x0, y0));
                        }
                        assert!(
                            fwd_inc.iter().rev().eq(rev_inc.iter()),
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

        assert_eq!(res, [(2, 3), (3, 3), (4, 3)]);
    }

    #[test]
    fn test_straight_vline() {
        let bi = Bresenham::new((2, 3), (2, 6));
        let res: Vec<_> = bi.collect();

        assert_eq!(res, [(2, 3), (2, 4), (2, 5)]);
    }

    #[test]
    fn test_degenerate() {
        let res: Vec<_> = Bresenham::new((3, 3), (3, 3)).collect();
        assert_eq!(res, []);
    }
}

