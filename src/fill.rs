//! Filled scanlines: solid horizontal spans plus anti-aliased edge points.

use crate::PointAa;

/// Horizontal span from column `x0` to `x1` on row `y`
///
/// A horizontal span was chosen over a vertical span because often times images
/// are stored in a row-major format, but it should actually be written in
/// whichever way goes with the locality of the image storage in cases where
/// that is relevant.
///
/// Invariant: `x0 <= x1`
/// Interval: Inclusive, `[start, end]`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub x0: isize,
    pub x1: isize,
    pub y: isize,
}

/// One fill instruction: a solid row or an anti-aliased edge pixel
///
/// Anti-aliased fills yield a mix of solid [`Span`]s and [`Point`]s carrying
/// edge coverage. Aliased fills yield [`Span`] directly via [`Fill`]'s
/// default item type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plot {
    /// A single pixel with anti-alias coverage (`255` fully on).
    Point(PointAa),
    /// A fully covered inclusive `[x0, x1]` run on one row.
    Span(Span),
}

/// Fill a shape with horizontal spans (and, for anti-aliased shapes, edge
/// points).
///
/// The iterator item defaults to [`Span`]. Anti-aliased fills implement
/// `Fill<Plot>` instead.
pub trait Fill<Item = Span> {
    /// Instructions covering the interior.
    fn fill(self) -> impl Iterator<Item = Item>;
}
