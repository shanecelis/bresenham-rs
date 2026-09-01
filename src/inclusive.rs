//! Inclusive `[start, end]` adapter for half-open line walkers.

/// Transform a half-open walker to an inclusive one.
pub trait Inclusive {
    /// Point or voxel type of the walker.
    type Item;

    /// Inclusive `[start, end]` pixels.
    fn inclusive(self) -> impl Iterator<Item = Self::Item>;
}
