//! Iterator-based Bresenham rasterizers
//!
//! [Bresenham's line drawing algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm) is a fast
//! integer algorithm to draw a line between two points. By default this crate
//! ships only that 2D line walker (`line`). Other primitives from
//! [Alois Zingl's notes](https://zingl.github.io/bresenham.html) are optional
//! Cargo features. It calculates coordinates without knowing anything about
//! drawing methods or surfaces.
//!
//! | Feature   | Types                                      |
//! |-----------|--------------------------------------------|
//! | `line`    | `Bresenham` (default)                      |
//! | `line3d`  | `Bresenham3d`                              |
//! | `circle`  | `Circle`                                   |
//! | `ellipse` | `Ellipse`, `EllipseRect`                   |
//! | `bezier`  | `QuadBezier`                               |
//! | `aa`      | `BresenhamAA`, `WideLine`, `QuadBezierAA` |
//!
//! Example:
//!
//! ```rust
//! # #[cfg(feature = "line")] {
//! extern crate bresenham;
//! use bresenham::Bresenham;
//!
//! fn main() {
//!     for (x, y) in Bresenham::new((0, 1), (6, 4)) {
//!         println!("{}, {}", x, y);
//!     }
//! }
//! # }
//! ```
//!
//! Will print:
//!
//! ```text
//! (0, 1)
//! (1, 1)
//! (2, 2)
//! (3, 2)
//! (4, 3)
//! (5, 3)
//! ```

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
extern crate std;

#[cfg(feature = "aa")]
mod aa;
#[cfg(feature = "bezier")]
mod bezier;
#[cfg(feature = "circle")]
mod circle;
#[cfg(feature = "ellipse")]
mod ellipse;
#[cfg(feature = "line")]
mod line;
#[cfg(feature = "line3d")]
mod line3d;
#[cfg(any(feature = "aa", feature = "bezier"))]
mod plot;

#[cfg(feature = "aa")]
#[cfg_attr(docsrs, doc(cfg(feature = "aa")))]
pub use aa::{AaPixel, BresenhamAA, WideLine, QuadBezierAA};
#[cfg(feature = "bezier")]
#[cfg_attr(docsrs, doc(cfg(feature = "bezier")))]
pub use bezier::QuadBezier;
#[cfg(feature = "circle")]
#[cfg_attr(docsrs, doc(cfg(feature = "circle")))]
pub use circle::Circle;
#[cfg(feature = "ellipse")]
#[cfg_attr(docsrs, doc(cfg(feature = "ellipse")))]
pub use ellipse::{Ellipse, EllipseRect};
#[cfg(feature = "line")]
#[cfg_attr(docsrs, doc(cfg(feature = "line")))]
pub use line::Bresenham;
#[cfg(feature = "line3d")]
#[cfg_attr(docsrs, doc(cfg(feature = "line3d")))]
pub use line3d::Bresenham3d;

/// Convenient typedef for two machine-sized integers
pub type Point = (isize, isize);

/// Convenient typedef for three machine-sized integers
#[cfg(feature = "line3d")]
#[cfg_attr(docsrs, doc(cfg(feature = "line3d")))]
pub type Point3 = (isize, isize, isize);

