#![doc = include_str!("../README.md")]
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
#[cfg(feature = "fill")]
mod fill;
#[cfg(feature = "inclusive")]
mod inclusive;
#[cfg(feature = "line")]
mod line;
#[cfg(feature = "line3d")]
mod line3d;

#[cfg(feature = "aa")]
#[cfg_attr(docsrs, doc(cfg(feature = "aa")))]
pub use aa::{AaPixel, LineAA, QuadBezierAA, WideLine};
#[cfg(feature = "bezier")]
#[cfg_attr(docsrs, doc(cfg(feature = "bezier")))]
pub use bezier::QuadBezier;
#[cfg(feature = "circle")]
#[cfg_attr(docsrs, doc(cfg(feature = "circle")))]
pub use circle::Circle;
#[cfg(feature = "ellipse")]
#[cfg_attr(docsrs, doc(cfg(feature = "ellipse")))]
pub use ellipse::{Ellipse, EllipseRect};
#[cfg(feature = "fill")]
#[cfg_attr(docsrs, doc(cfg(feature = "fill")))]
pub use fill::{Fill, Span};
#[cfg(feature = "inclusive")]
#[cfg_attr(docsrs, doc(cfg(feature = "inclusive")))]
pub use inclusive::Inclusive;
#[cfg(feature = "line")]
#[cfg_attr(docsrs, doc(cfg(feature = "line")))]
pub use line::{Bresenham, Line};
#[cfg(feature = "line3d")]
#[cfg_attr(docsrs, doc(cfg(feature = "line3d")))]
pub use line3d::Line3d;

/// Convenient typedef for two machine-sized integers
pub type Point = (isize, isize);

/// Convenient typedef for three machine-sized integers
#[cfg(feature = "line3d")]
#[cfg_attr(docsrs, doc(cfg(feature = "line3d")))]
pub type Point3 = (isize, isize, isize);
