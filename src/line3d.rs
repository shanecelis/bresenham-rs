//! 3D line from Alois Zingl's `plotLine3d`.

use crate::Point3;

/// 3D line-drawing iterator. Half-open: yields `[start, end)`.
pub struct Line3d {
    x: isize,
    y: isize,
    z: isize,
    dx: isize,
    dy: isize,
    dz: isize,
    sx: isize,
    sy: isize,
    sz: isize,
    x_err: isize,
    y_err: isize,
    z_err: isize,
    dm: isize,
    remaining: isize,
}

impl Line3d {
    /// Yields every voxel from `start` toward `end`, excluding `end`
    /// (`[start, end)`).
    #[inline]
    pub fn new(start: Point3, end: Point3) -> Self {
        let (x0, y0, z0) = start;
        let (x1, y1, z1) = end;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let dz = (z1 - z0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let sz = if z0 < z1 { 1 } else { -1 };
        let dm = dx.max(dy).max(dz);

        Line3d {
            x: x0,
            y: y0,
            z: z0,
            dx,
            dy,
            dz,
            sx,
            sy,
            sz,
            x_err: dm / 2,
            y_err: dm / 2,
            z_err: dm / 2,
            dm,
            remaining: dm,
        }
    }
}

impl Iterator for Line3d {
    type Item = Point3;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;

        let p = (self.x, self.y, self.z);

        self.x_err -= self.dx;
        if self.x_err < 0 {
            self.x_err += self.dm;
            self.x += self.sx;
        }
        self.y_err -= self.dy;
        if self.y_err < 0 {
            self.y_err += self.dm;
            self.y += self.sy;
        }
        self.z_err -= self.dz;
        if self.z_err < 0 {
            self.z_err += self.dm;
            self.z += self.sz;
        }

        Some(p)
    }
}

#[cfg(test)]
mod tests {
    use super::Line3d;
    use std::vec::Vec;

    #[test]
    fn test_line3d() {
        let res: Vec<_> = Line3d::new((0, 0, 0), (2, 1, 0)).collect();
        assert_eq!(res, [(0, 0, 0), (1, 0, 0)]);

        let res: Vec<_> = Line3d::new((0, 0, 0), (3, 3, 3)).collect();
        assert_eq!(res, [(0, 0, 0), (1, 1, 1), (2, 2, 2)]);

        let res: Vec<_> = Line3d::new((1, 2, 3), (1, 2, 3)).collect();
        assert_eq!(res, []);
    }
}
