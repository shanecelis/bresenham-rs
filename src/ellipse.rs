//! Axis-aligned ellipses from Alois Zingl's `plotEllipse` and `plotEllipseRect`.

use crate::Point;

enum EllipsePhase {
    Main { quad: u8 },
    Tip { which: u8 },
}

/// Iterator over the pixels of an axis-aligned ellipse given a center and radii.
pub struct Ellipse {
    xm: isize,
    ym: isize,
    a: isize,
    b: isize,
    x: isize,
    y: isize,
    err: i64,
    phase: EllipsePhase,
    done: bool,
}

impl Ellipse {
    /// Ellipse centered at `center` with horizontal radius `a` and vertical
    /// radius `b`. Negative radii are treated as their absolute value.
    #[inline]
    pub fn new(center: Point, a: isize, b: isize) -> Self {
        let a = a.abs();
        let b = b.abs();
        let x = -a;
        let e2 = (b as i64) * (b as i64);
        let err = (x as i64) * (2 * e2 + x as i64) + e2;

        Ellipse {
            xm: center.0,
            ym: center.1,
            a,
            b,
            x,
            y: 0,
            err,
            phase: EllipsePhase::Main { quad: 0 },
            done: false,
        }
    }

    /// Horizontal chords of the filled ellipse: `f(x0, x1, y)`.
    #[inline]
    pub fn for_each_hline<F: FnMut(isize, isize, isize)>(mut self, mut f: F) {
        let a2 = (self.a as i64) * (self.a as i64);
        let b2 = (self.b as i64) * (self.b as i64);
        loop {
            let x0 = self.xm + self.x;
            let x1 = self.xm - self.x;
            f(x0, x1, self.ym + self.y);
            if self.y != 0 {
                f(x0, x1, self.ym - self.y);
            }
            let e2 = 2 * self.err;
            if e2 >= (self.x * 2 + 1) as i64 * b2 {
                self.x += 1;
                self.err += (self.x * 2 + 1) as i64 * b2;
            }
            if e2 <= (self.y * 2 + 1) as i64 * a2 {
                self.y += 1;
                self.err += (self.y * 2 + 1) as i64 * a2;
            }
            if self.x > 0 {
                break;
            }
        }
        while self.y < self.b {
            self.y += 1;
            f(self.xm, self.xm, self.ym + self.y);
            f(self.xm, self.xm, self.ym - self.y);
        }
    }
}

impl Iterator for Ellipse {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.phase {
            EllipsePhase::Main { quad } => {
                let p = match quad {
                    0 => (self.xm - self.x, self.ym + self.y),
                    1 => (self.xm + self.x, self.ym + self.y),
                    2 => (self.xm + self.x, self.ym - self.y),
                    3 => (self.xm - self.x, self.ym - self.y),
                    _ => unreachable!(),
                };

                if quad < 3 {
                    self.phase = EllipsePhase::Main { quad: quad + 1 };
                } else {
                    let a2 = (self.a as i64) * (self.a as i64);
                    let b2 = (self.b as i64) * (self.b as i64);
                    let e2 = 2 * self.err;
                    if e2 >= (self.x * 2 + 1) as i64 * b2 {
                        self.x += 1;
                        self.err += (self.x * 2 + 1) as i64 * b2;
                    }
                    if e2 <= (self.y * 2 + 1) as i64 * a2 {
                        self.y += 1;
                        self.err += (self.y * 2 + 1) as i64 * a2;
                    }
                    if self.x <= 0 {
                        self.phase = EllipsePhase::Main { quad: 0 };
                    } else {
                        self.phase = EllipsePhase::Tip { which: 0 };
                    }
                }

                Some(p)
            }
            EllipsePhase::Tip { which } => {
                // `while (y++ < b)` — increment first, then plot if the old y was < b.
                if which == 0 {
                    self.y += 1;
                    if self.y > self.b {
                        self.done = true;
                        return None;
                    }
                }

                let p = if which == 0 {
                    (self.xm, self.ym + self.y)
                } else {
                    (self.xm, self.ym - self.y)
                };

                self.phase = EllipsePhase::Tip {
                    which: if which == 0 { 1 } else { 0 },
                };
                Some(p)
            }
        }
    }
}

enum RectPhase {
    Main { quad: u8 },
    Tip { which: u8 },
}

/// Iterator over an axis-aligned ellipse inscribed in a rectangle.
pub struct EllipseRect {
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    a: i64,
    b: i64,
    b1: i64,
    dx: i64,
    dy: i64,
    err: i64,
    phase: RectPhase,
    done: bool,
}

impl EllipseRect {
    /// Ellipse filling the rectangle with opposite corners `p0` and `p1`.
    #[inline]
    pub fn new(p0: Point, p1: Point) -> Self {
        let (mut x0, mut y0) = p0;
        let (mut x1, y1) = p1;
        let a = (x1 - x0).abs() as i64;
        let b = (y1 - y0).abs() as i64;
        let b1 = b & 1;
        let dx = 4 * (1 - a) * b * b;
        let dy = 4 * (b1 + 1) * a * a;
        let err = dx + dy + b1 * a * a;

        if x0 > x1 {
            x0 = x1;
            x1 += a as isize;
        }
        if y0 > y1 {
            y0 = y1;
        }
        y0 += ((b + 1) / 2) as isize;
        let y1 = y0 - b1 as isize;
        let a = 8 * a * a;
        let b1 = 8 * b * b;

        EllipseRect {
            x0,
            y0,
            x1,
            y1,
            a,
            b,
            b1,
            dx,
            dy,
            err,
            phase: RectPhase::Main { quad: 0 },
            done: false,
        }
    }

    #[inline]
    fn points4(&self) -> [Point; 4] {
        [
            (self.x1, self.y0),
            (self.x0, self.y0),
            (self.x0, self.y1),
            (self.x1, self.y1),
        ]
    }

    #[inline]
    fn tip4(&self) -> [Point; 4] {
        [
            (self.x0 - 1, self.y0),
            (self.x1 + 1, self.y0),
            (self.x0 - 1, self.y1),
            (self.x1 + 1, self.y1),
        ]
    }

    #[inline]
    fn advance_main(&mut self) {
        let e2 = 2 * self.err;
        if e2 <= self.dy {
            self.y0 += 1;
            self.y1 -= 1;
            self.dy += self.a;
            self.err += self.dy;
        }
        if e2 >= self.dx || 2 * self.err > self.dy {
            self.x0 += 1;
            self.x1 -= 1;
            self.dx += self.b1;
            self.err += self.dx;
        }
        if self.x0 > self.x1 {
            self.phase = RectPhase::Tip { which: 0 };
        }
    }

    /// Call `f` with every outline pixel. Faster than the iterator when inlined.
    #[inline]
    pub fn for_each<F: FnMut(Point)>(mut self, mut f: F) {
        while let RectPhase::Main { .. } = self.phase {
            let pts = self.points4();
            f(pts[0]);
            f(pts[1]);
            f(pts[2]);
            f(pts[3]);
            self.advance_main();
        }
        while (self.y0 - self.y1) as i64 <= self.b {
            let pts = self.tip4();
            f(pts[0]);
            f(pts[1]);
            self.y0 += 1;
            f(pts[2]);
            f(pts[3]);
            self.y1 -= 1;
        }
    }

    /// Horizontal chords of the filled ellipse: `f(x0, x1, y)`.
    ///
    /// Each step joins the left and right pixels at the current `y0` / `y1`.
    #[inline]
    pub fn for_each_hline<F: FnMut(isize, isize, isize)>(mut self, mut f: F) {
        while let RectPhase::Main { .. } = self.phase {
            f(self.x0, self.x1, self.y0);
            if self.y0 != self.y1 {
                f(self.x0, self.x1, self.y1);
            }
            self.advance_main();
        }
        while (self.y0 - self.y1) as i64 <= self.b {
            f(self.x0 - 1, self.x1 + 1, self.y0);
            self.y0 += 1;
            f(self.x0 - 1, self.x1 + 1, self.y1);
            self.y1 -= 1;
        }
    }
}

impl Iterator for EllipseRect {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.phase {
            RectPhase::Main { quad } => {
                let p = self.points4()[quad as usize];

                if quad < 3 {
                    self.phase = RectPhase::Main { quad: quad + 1 };
                } else {
                    self.phase = RectPhase::Main { quad: 0 };
                    self.advance_main();
                }

                Some(p)
            }
            RectPhase::Tip { which } => {
                // `while (y0 - y1 <= b)` — only test at the start of a 4-pixel group.
                if which == 0 && (self.y0 - self.y1) as i64 > self.b {
                    self.done = true;
                    return None;
                }

                let p = self.tip4()[which as usize];

                if which < 3 {
                    if which == 1 {
                        self.y0 += 1;
                    }
                    self.phase = RectPhase::Tip { which: which + 1 };
                } else {
                    self.y1 -= 1;
                    self.phase = RectPhase::Tip { which: 0 };
                }

                Some(p)
            }
        }
    }
}
