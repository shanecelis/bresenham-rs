//! Inclusive `[start, end]` adapter for half-open line walkers.

use crate::Point;

/// A half-open walker that can also yield `end`.
pub trait Inclusive {
    /// Inclusive `[start, end]` pixels.
    fn inclusive(self) -> impl Iterator<Item = Point>;
}
