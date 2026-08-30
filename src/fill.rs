//! Filled scanlines: one inclusive horizontal span per distinct row.

/// Inclusive horizontal span `[x0, x1]` at `y` (`x0 <= x1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HLine {
    pub x0: isize,
    pub x1: isize,
    pub y: isize,
}

/// A shape that can be filled as horizontal spans, one per distinct row.
pub trait Fill {
    /// Inclusive `[x0, x1]` chords covering the interior.
    fn fill(self) -> impl Iterator<Item = HLine>;
}
