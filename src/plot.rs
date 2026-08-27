//! Zingl's integer line walker, used as a fallback by the curve iterators.

#[cfg(feature = "bezier")]
use crate::Point;

/// Inclusive 2D line from Alois Zingl's `plotLine` (`start..=end`).
#[cfg(feature = "bezier")]
pub(crate) struct PlotLine {
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    dx: isize,
    dy: isize,
    sx: isize,
    sy: isize,
    err: isize,
    done: bool,
}

#[cfg(feature = "bezier")]
impl PlotLine {
    pub(crate) fn new(start: Point, end: Point) -> Self {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        PlotLine {
            x0,
            y0,
            x1,
            y1,
            dx,
            dy,
            sx,
            sy,
            err: dx + dy,
            done: false,
        }
    }
}

#[cfg(feature = "bezier")]
impl Iterator for PlotLine {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let p = (self.x0, self.y0);
        let e2 = 2 * self.err;

        if e2 >= self.dy {
            if self.x0 == self.x1 {
                self.done = true;
                return Some(p);
            }
            self.err += self.dy;
            self.x0 += self.sx;
        }

        if e2 <= self.dx {
            if self.y0 == self.y1 {
                self.done = true;
                return Some(p);
            }
            self.err += self.dx;
            self.y0 += self.sy;
        }

        Some(p)
    }
}

#[cfg(any(feature = "aa", feature = "bezier"))]
#[inline]
pub(crate) fn abs_f64(v: f64) -> f64 {
    if v < 0.0 {
        -v
    } else {
        v
    }
}

#[cfg(feature = "aa")]
#[inline]
pub(crate) fn min_f64(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

#[cfg(feature = "aa")]
#[inline]
pub(crate) fn max_f64(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

/// `no_std` `floor` for values that fit in `i64`.
#[cfg(feature = "bezier")]
#[inline]
pub(crate) fn floor_f64(v: f64) -> f64 {
    let t = v as i64 as f64;
    if v < t {
        t - 1.0
    } else {
        t
    }
}

/// C `floor(v + 0.5)` — half-up toward +∞.
#[cfg(feature = "bezier")]
#[inline]
pub(crate) fn iround(v: f64) -> isize {
    floor_f64(v + 0.5) as isize
}

/// Newton–Raphson square root for `no_std`.
#[cfg(feature = "aa")]
#[inline]
pub(crate) fn sqrt_f64(v: f64) -> f64 {
    if v <= 0.0 {
        return 0.0;
    }
    let mut x = v;
    for _ in 0..16 {
        x = 0.5 * (x + v / x);
    }
    x
}
