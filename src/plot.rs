//! Shared `no_std` float helpers for curve and anti-aliased iterators.

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
