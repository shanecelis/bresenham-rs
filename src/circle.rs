//! Midpoint circle from Alois Zingl's `plotCircle`.

use crate::Point;

/// Iterator over the pixels of an axis-aligned circle. Closed outline.
pub struct Circle {
    xm: isize,
    ym: isize,
    x: isize,
    y: isize,
    err: isize,
    quad: u8,
    done: bool,
}

impl Circle {
    /// Closed circle centered at `center` with the given `radius`.
    ///
    /// A radius of `0` yields the center point once. Negative radii are treated
    /// as their absolute value.
    #[inline]
    pub fn new(center: Point, radius: isize) -> Self {
        let r = radius.abs();
        if r == 0 {
            return Circle {
                xm: center.0,
                ym: center.1,
                x: 0,
                y: 0,
                err: 0,
                quad: 4,
                done: false,
            };
        }

        Circle {
            xm: center.0,
            ym: center.1,
            x: -r,
            y: 0,
            err: 2 - 2 * r,
            quad: 0,
            done: false,
        }
    }

    #[inline]
    fn points4(&self) -> [Point; 4] {
        [
            (self.xm - self.x, self.ym + self.y),
            (self.xm - self.y, self.ym - self.x),
            (self.xm + self.x, self.ym - self.y),
            (self.xm + self.y, self.ym + self.x),
        ]
    }

    #[inline]
    fn advance(&mut self) {
        let r = self.err;
        if r <= self.y {
            self.y += 1;
            self.err += self.y * 2 + 1;
        }
        if r > self.x || self.err > self.y {
            self.x += 1;
            self.err += self.x * 2 + 1;
        }
        if self.x >= 0 {
            self.done = true;
        }
    }

    /// Call `f` with every outline pixel. Faster than the iterator when the
    /// body can be inlined (four plots per step, no per-pixel `next`).
    #[inline]
    pub fn for_each<F: FnMut(Point)>(mut self, mut f: F) {
        if self.quad == 4 {
            f((self.xm, self.ym));
            return;
        }
        while !self.done {
            let pts = self.points4();
            f(pts[0]);
            f(pts[1]);
            f(pts[2]);
            f(pts[3]);
            self.advance();
        }
    }

    /// Horizontal chords of the filled circle: `f(x0, x1, y)`.
    ///
    /// Each step mirrors the current point across the y-axis and draws that
    /// span at `+y` and `-y`.
    #[inline]
    pub fn for_each_hline<F: FnMut(isize, isize, isize)>(mut self, mut f: F) {
        if self.quad == 4 {
            f(self.xm, self.xm, self.ym);
            return;
        }
        while !self.done {
            let x0 = self.xm + self.x;
            let x1 = self.xm - self.x;
            f(x0, x1, self.ym + self.y);
            if self.y != 0 {
                f(x0, x1, self.ym - self.y);
            }
            self.advance();
        }
    }
}

impl Iterator for Circle {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // r == 0: a single center pixel.
        if self.quad == 4 {
            self.done = true;
            return Some((self.xm, self.ym));
        }

        let p = self.points4()[self.quad as usize];

        if self.quad < 3 {
            self.quad += 1;
        } else {
            self.quad = 0;
            self.advance();
        }

        Some(p)
    }
}

#[cfg(test)]
mod tests {
    use super::Circle;
    use std::vec::Vec;

    #[test]
    fn test_circle() {
        let res: Vec<_> = Circle::new((0, 0), 0).collect();
        assert_eq!(res, [(0, 0)]);

        let res: Vec<_> = Circle::new((0, 0), 1).collect();
        assert_eq!(res, [(1, 0), (0, 1), (-1, 0), (0, -1)]);

        let res: Vec<_> = Circle::new((5, 5), 2).collect();
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

        let res: Vec<_> = Circle::new((0, 0), 4).collect();
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
    fn test_circle_for_each_matches_iter() {
        for r in 0..16 {
            let a: Vec<_> = Circle::new((3, -2), r).collect();
            let mut b = Vec::new();
            Circle::new((3, -2), r).for_each(|p| b.push(p));
            assert_eq!(a, b, "r={r}");
        }
    }
}

