# bresenham

Iterator-based
[Bresenham](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)'s line
drawing algorithm.

Bresenham's line drawing algorithm is a fast algorithm to draw a line between
two points, without any overdraw. This crate implements the fast integer
variant, using an iterator-based approach for flexibility. Most, if not all,
overhead should evaporate when inlined by the compiler. It calculates
coordinates without knowing anything about drawing methods or surfaces.

## Example

```rust
for (x, y) in bresenham::Line::new((0, 1), (6, 4)) {
    println!("{}, {}", x, y);
}
```

```text
(0, 1)
(1, 1)
(2, 2)
(3, 2)
(4, 3)
(5, 3)
```

## Bresenham Variant Notes

The lines are drawn on a half-open interval `[start, end)`: the `start` point is
included, but the `end` point is not. This allows one to chain multiple lines
together without any overdraw.

This particular implementation of Bresenham breaks ties in quadrants such that a
line drawn from `(A, B)` and from `(B, A)` will share the same points,
neglecting their extreme points due to it being half-open.

## Alois Zingl's Additions

[Alois Zingl's notes](https://zingl.github.io/bresenham.html) on Bresenham were
used to add other shapes, available as optional Cargo features.

## Cargo Features

| Feature          | Types                                | Interval                 |
|------------------|--------------------------------------|--------------------------|
| `line` (default) | `Line`                               | half-open `[start, end)` |
| `line3d`         | `Line3d`                             | half-open `[start, end)` |
| `circle`         | `Circle`                             | closed outline           |
| `ellipse`        | `Ellipse`, `EllipseRect`             | closed outline           |
| `bezier`         | `QuadBezier`                         | inclusive `[start, end]` |
| `aa`             | `LineAA`, `WideLine`, `QuadBezierAA` | inclusive `[start, end]` |

## Boundaries

The boundaries half-open or inclusive were chosen for performance,
composability, and ease to convert one to the other, e.g., a line may be made
inclusive by drawing the line and then plotting its `end` argument. An inclusive
interval may be created half-open by dropping its last element. This library
does not provide those conveniences because they do incur a small performance
penalty, and I am of the opinion that performance degradation should not be made
convenient.

## Fill

The `fill` Cargo feature adds `Fillable::fill` on `Circle`, `Ellipse`, and
`EllipseRect`. It yields one `HLine` per distinct row: a horizontal span from
the leftmost filled pixel to the rightmost. Enable it together with `circle`
and/or `ellipse`.
