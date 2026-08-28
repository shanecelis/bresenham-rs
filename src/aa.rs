//! Anti-aliased primitives from Alois Zingl's `plotLineAA`, `plotQuadBezierSegAA`,
//! and `plotLineWidth`.
//!
//! Coverage is inverted from Zingl's `setPixelAA`: `255` is fully on the curve,
//! `0` is fully off.

use crate::Point;

/// A pixel plus coverage (`255` = fully on, `0` = fully off).
pub type AaPixel = (Point, u8);

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

/// Anti-aliased 2D line (Zingl `plotLineAA`). Inclusive: `start..=end`.
pub struct LineAA {
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    dx: isize,
    dy: isize,
    sx: isize,
    sy: isize,
    err: isize,
    ed: isize,
    pending: [AaPixel; 3],
    pending_len: u8,
    pending_i: u8,
    done: bool,
}

impl LineAA {
    /// Inclusive anti-aliased line (`start..=end`).
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
            libm::sqrt((dx * dx + dy * dy) as f64) as isize
        };
        let ed = if ed == 0 { 1 } else { ed };

        LineAA {
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

    fn pop_pending(&mut self) -> Option<AaPixel> {
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

impl Iterator for LineAA {
    type Item = AaPixel;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.pop_pending() {
            return Some(p);
        }
        if self.done {
            return None;
        }

        let fade = coverage_i(255 * (self.err - self.dx + self.dy).abs() / self.ed);
        self.push((self.x0, self.y0), fade);

        let e2 = self.err;
        let x2 = self.x0;
        if 2 * e2 >= -self.dx {
            if self.x0 == self.x1 {
                self.done = true;
                return self.pop_pending();
            }
            if e2 + self.dy < self.ed {
                self.push(
                    (self.x0, self.y0 + self.sy),
                    coverage_i(255 * (e2 + self.dy) / self.ed),
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
            if self.dx - e2 < self.ed {
                self.push(
                    (x2 + self.sx, self.y0),
                    coverage_i(255 * (self.dx - e2) / self.ed),
                );
            }
            self.err += self.dx;
            self.y0 += self.sy;
        }

        self.pop_pending()
    }
}

enum LwPhase {
    Center,
    XPerp { e2: isize, y2: isize },
    YGate { e2: isize, x2: isize },
    YPerp { e2: isize, x2: isize },
}

/// Anti-aliased line of a given pixel width (Zingl `plotWideLine`).
/// Inclusive: `start..=end`.
pub struct WideLine {
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

impl WideLine {
    /// Inclusive anti-aliased line (`start..=end`) with width `wd`.
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
            libm::sqrt((dx * dx + dy * dy) as f64)
        };

        WideLine {
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

impl Iterator for WideLine {
    type Item = AaPixel;

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

enum BezierAaState {
    Curve,
    Line(LineAA),
    Done,
}

/// Anti-aliased quadratic Bézier (Zingl `plotQuadBezierSegAA`). Inclusive: `p0..=p2`.
///
/// Like the C original, the gradient sign must not change along the segment;
/// if it does, the remainder is finished with an anti-aliased line.
pub struct QuadBezierAA {
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
    pending: [AaPixel; 3],
    pending_len: u8,
    pending_i: u8,
}

impl QuadBezierAA {
    /// Inclusive quadratic Bézier (`p0..=p2`) with control point `p1`.
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
            return QuadBezierAA {
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

        QuadBezierAA {
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
            state: BezierAaState::Line(LineAA::new((x0, y0), (x2, y2))),
            pending: [((0, 0), 0); 3],
            pending_len: 0,
            pending_i: 0,
        }
    }

    fn push(&mut self, p: Point, fade: u8) {
        self.pending[self.pending_len as usize] = (p, fade);
        self.pending_len += 1;
    }

    fn pop_pending(&mut self) -> Option<AaPixel> {
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
        let fade =
            coverage_f(255.0 * (self.err - self.dx - self.dy - self.xy as f64).abs() / ed);
        self.push((self.x0, self.y0), fade);

        if self.x0 == self.x2 || self.y0 == self.y2 {
            self.state =
                BezierAaState::Line(LineAA::new((self.x0, self.y0), (self.x2, self.y2)));
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
                self.push(
                    (x1 + self.sx, self.y0),
                    coverage_f(255.0 * cur.abs() / ed),
                );
            }
            self.y0 += self.sy;
            self.dy -= self.xy as f64;
            self.dx += self.xx as f64;
            self.err += self.dx;
        }

        if !(self.dy < self.dx) {
            self.state =
                BezierAaState::Line(LineAA::new((self.x0, self.y0), (self.x2, self.y2)));
        }
    }
}

impl Iterator for QuadBezierAA {
    type Item = AaPixel;

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
    use super::{LineAA, WideLine, QuadBezierAA};
    use std::vec::Vec;

    #[test]
    fn test_line_aa() {
        let res: Vec<_> = LineAA::new((0, 0), (4, 0)).collect();
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

        let res: Vec<_> = LineAA::new((0, 1), (6, 4)).collect();
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

        let res: Vec<_> = LineAA::new((0, 0), (3, 3)).collect();
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
    fn test_wide_line() {
        let res: Vec<_> = WideLine::new((0, 0), (4, 0), 1.0).collect();
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

        let res: Vec<_> = WideLine::new((0, 0), (5, 2), 3.0).collect();
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

    #[test]
    fn test_quad_bezier_aa() {
        let res: Vec<_> = QuadBezierAA::new((0, 0), (1, 3), (4, 3)).collect();
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

