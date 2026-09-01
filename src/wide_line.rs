//! Anti-aliased thick line from Alois Zingl's `plotLineWidth`.
//!
//! Coverage is inverted from Zingl's `setPixelAA`: `255` is fully on the curve,
//! `0` is fully off.

use crate::{Point, PointAa};

fn coverage_f(zingl_fade: f64) -> u8 {
    255 - if zingl_fade <= 0.0 {
        0
    } else if zingl_fade >= 255.0 {
        255
    } else {
        zingl_fade as u8
    }
}

enum LwPhase {
    Center,
    XPerp { e2: isize, y2: isize },
    YGate { e2: isize, x2: isize },
    YPerp { e2: isize, x2: isize },
}

/// Anti-aliased line of a given pixel width
///
/// Inclusive: `[start, end]`.
pub struct WideLineAa {
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    dx: isize,
    dy: isize,
    sx: isize,
    sy: isize,
    err: isize,
    ed: f64,
    wd: f64,
    phase: LwPhase,
    done: bool,
}

impl WideLineAa {
    /// Inclusive anti-aliased line (`[start, end]`) with width `wd`.
    pub fn new(start: Point, end: Point, wd: f32) -> Self {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let ed = if dx + dy == 0 {
            1.0
        } else {
            // Look for the [core::f64::math::sqrt] to become stable so you can
            // remove this.
            libm::sqrt((dx * dx + dy * dy) as f64)
        };

        WideLineAa {
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
            wd: (wd as f64 + 1.0) / 2.0,
            phase: LwPhase::Center,
            done: false,
        }
    }

    fn color(&self, dist: f64) -> u8 {
        coverage_f(255.0 * (dist.abs() / self.ed - self.wd + 1.0))
    }
}

impl Iterator for WideLineAa {
    type Item = PointAa;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.done {
            match self.phase {
                LwPhase::Center => {
                    let fade = self.color((self.err - self.dx + self.dy) as f64);
                    let p = (self.x0, self.y0);
                    let e2 = self.err;
                    let x2 = self.x0;
                    if 2 * e2 >= -self.dx {
                        self.phase = LwPhase::XPerp {
                            e2: e2 + self.dy,
                            y2: self.y0,
                        };
                    } else {
                        self.phase = LwPhase::YGate { e2, x2 };
                    }
                    return Some((p, fade));
                }
                LwPhase::XPerp { e2, y2 } => {
                    if (e2 as f64) < self.ed * self.wd && (self.y1 != y2 || self.dx > self.dy) {
                        let y2 = y2 + self.sy;
                        let fade = self.color(e2 as f64);
                        self.phase = LwPhase::XPerp {
                            e2: e2 + self.dx,
                            y2,
                        };
                        return Some(((self.x0, y2), fade));
                    }
                    if self.x0 == self.x1 {
                        self.done = true;
                    } else {
                        let e2 = self.err;
                        self.err -= self.dy;
                        self.x0 += self.sx;
                        self.phase = LwPhase::YGate {
                            e2,
                            x2: self.x0 - self.sx,
                        };
                    }
                }
                LwPhase::YGate { e2, x2 } => {
                    if 2 * e2 <= self.dy {
                        self.phase = LwPhase::YPerp {
                            e2: self.dx - e2,
                            x2,
                        };
                    } else {
                        self.phase = LwPhase::Center;
                    }
                }
                LwPhase::YPerp { e2, x2 } => {
                    if (e2 as f64) < self.ed * self.wd && (self.x1 != x2 || self.dx < self.dy) {
                        let x2 = x2 + self.sx;
                        let fade = self.color(e2 as f64);
                        self.phase = LwPhase::YPerp {
                            e2: e2 + self.dy,
                            x2,
                        };
                        return Some(((x2, self.y0), fade));
                    }
                    if self.y0 == self.y1 {
                        self.done = true;
                    } else {
                        self.err += self.dx;
                        self.y0 += self.sy;
                        self.phase = LwPhase::Center;
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::WideLineAa;
    use std::vec::Vec;

    #[test]
    fn test_wide_line() {
        let res: Vec<_> = WideLineAa::new((0, 0), (4, 0), 1.0).collect();
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

        let res: Vec<_> = WideLineAa::new((0, 0), (5, 2), 3.0).collect();
        assert_eq!(
            res,
            [
                ((0, 0), 255),
                ((0, 1), 255),
                ((0, 2), 37),
                ((1, 0), 255),
                ((1, 1), 255),
                ((1, 2), 132),
                ((2, 0), 255),
                ((3, 0), 226),
                ((4, 0), 132),
                ((5, 0), 37),
                ((2, 1), 255),
                ((2, 2), 226),
                ((3, 1), 255),
                ((3, 2), 255),
                ((3, 3), 84),
                ((4, 1), 255),
                ((4, 2), 255),
                ((4, 3), 179),
                ((5, 1), 255),
                ((5, 2), 255),
                ((5, 3), 255),
                ((5, 4), 37)
            ]
        );
    }
}
