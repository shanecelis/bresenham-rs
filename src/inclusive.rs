//! Inclusive `[start, end]` adapter for half-open line walkers.

/// A half-open walker that can also yield `end`.
pub trait Inclusive {
    /// Point or voxel type of the walker.
    type Item;

    /// Inclusive `[start, end]` pixels.
    fn inclusive(self) -> impl Iterator<Item = Self::Item>;
}
