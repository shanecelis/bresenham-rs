//! Anti-aliased quadratic Bézier from Alois Zingl's `plotQuadBezierSegAA`.
//!
//! Coverage is inverted from Zingl's `setPixelAA`: `255` is fully on the curve,
//! `0` is fully off.

use crate::{LineAa, Point, PointAa};

fn coverage_f(zingl_fade: f64) -> u8 {
    255 - if zingl_fade <= 0.0 {
        0
    } else if zingl_fade >= 255.0 {
        255
    } else {
        zingl_fade as u8
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
    use super::QuadBezierAa;
    use std::vec::Vec;

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
