//! Filled scanlines: one inclusive horizontal span per distinct row.

/// Inclusive horizontal span `[x0, x1]` at `y`.
///
/// A horizontal span was chosen over a vertical span because often times images
/// are stored in a row-major format, but it should actually be written in
/// whichever way goes with the locality of the image storage in cases where
/// that is relevant.
///
/// Invariant: `x0 <= x1`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub x0: isize,
    pub x1: isize,
    pub y: isize,
}

/// A shape that can be filled as horizontal spans, one per distinct row.
pub trait Fill {
    /// Inclusive `[x0, x1]` chords covering the interior.
    fn fill(self) -> impl Iterator<Item = Span>;
}
