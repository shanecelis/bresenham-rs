//! 3D Bresenham line from Alois Zingl's `plotLine3d`.

use crate::Point3;

/// Inclusive 3D line-drawing iterator.
pub struct Bresenham3d {
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
    done: bool,
}

impl Bresenham3d {
    /// Yields every voxel from `start` through `end`, inclusive.
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

        Bresenham3d {
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
            done: false,
        }
    }
}

impl Iterator for Bresenham3d {
    type Item = Point3;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let p = (self.x, self.y, self.z);

        if self.remaining == 0 {
            self.done = true;
            return Some(p);
        }
        self.remaining -= 1;

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
