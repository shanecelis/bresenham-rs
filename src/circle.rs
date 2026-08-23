//! Midpoint circle from Alois Zingl's `plotCircle`.

use crate::Point;

/// Iterator over the pixels of an axis-aligned circle.
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
    /// Circle centered at `center` with the given `radius`.
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

        let p = match self.quad {
            0 => (self.xm - self.x, self.ym + self.y),
            1 => (self.xm - self.y, self.ym - self.x),
            2 => (self.xm + self.x, self.ym - self.y),
            3 => (self.xm + self.y, self.ym + self.x),
            _ => unreachable!(),
        };

        if self.quad < 3 {
            self.quad += 1;
        } else {
            self.quad = 0;
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

        Some(p)
    }
}
