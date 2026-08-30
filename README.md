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

By default lines are drawn on a half-open interval `[start, end)`: the `start`
point is included, but the `end` point is not. This allows one to chain multiple
lines together without any overdraw. However, one can opt-in to the "inclusive" Cargo feature to get an `line.inclusive()` iterator.

This particular implementation of Bresenham breaks ties in quadrants such that
an inclusive line drawn from `(A, B)` and from `(B, A)` will cover the same
points.

## Alois Zingl's Additions

[Alois Zingl's notes](https://zingl.github.io/bresenham.html) on Bresenham were
used to add other shapes, which are available as optional Cargo features.

## Shapes

<img width="512" height="384" alt="demo" src="https://github.com/user-attachments/assets/f24515b4-24d9-41bd-8c2b-69e0b0db6af8" />

Numbers match the digit shown in the demo.

| Demo | Shape              | Type                         | Interval                 | Feature          |
|------|--------------------|------------------------------|--------------------------|------------------|
| 0    | Line               | `Line`                       | half-open `[start, end)` | `line` (default) |
| 1    | Anti-aliased line  | `LineAA`                     | inclusive `[start, end]` | `aa`             |
| 2    | Circle             | `Circle`                     | closed outline           | `circle`         |
| 3    | Ellipse            | `EllipseRect`                | closed outline           | `ellipse`        |
| 4    | Quadratic Bézier   | `QuadBezier`                 | inclusive `[start, end]` | `bezier`         |
| 5    | Anti-aliased Bézier| `QuadBezierAA`               | inclusive `[start, end]` | `aa`             |
| 6    | Wide line          | `WideLine`                   | inclusive `[start, end]` | `aa`             |
| 7    | Filled circle      | `Circle` + `Fill`            | scanlines                | `circle`, `fill` |

`Line3d` (`line3d`) and center-and-radii `Ellipse` (`ellipse`) are also available; they are not in the demo.

## Demo

The WASM demo shows a 64×48 canvas that autoplays the shapes above. 

```sh
cd demo
trunk serve; # Open the URL trunk prints.
```

Left-drag draws the current shape between two points. Right-click stops autoplay
and advances to the next shape. After a few seconds of inactivity, autoplay
resumes.

## Boundaries

The boundaries half-open or inclusive were chosen for performance,
composability, and ease to convert one to the other. Converting from half-open
to inclusive is furnished by the `Inclusive` trait. An inclusive interval may be
created half-open by dropping its last point; however, this library does not
provide that convenience because it incurs a small performance penalty per step,
and the author maintains that performance degradation should not be made
convenient.

## Fill

The `fill` Cargo feature adds `Fill::fill` on `Circle`, `Ellipse`, and
`EllipseRect`. It yields one `HLine` per distinct row: a horizontal span from
the leftmost filled pixel to the rightmost. Enable it together with `circle`
and/or `ellipse`.

## Inclusive

The `inclusive` Cargo feature adds the `Inclusive` trait on `Line` and `Line3d`,
which returns an iterator that includes the end point making the interval
inclusive `[start, end]` instead of half-open. This approach
follows [indubitablement2's
work](https://github.com/indubitablement2/bresenham-rs), which is careful not
to incur any runtime penalty.
 
### Example

```rust
for (x, y) in bresenham::Line::new((0, 1), (6, 4)).inclusive() {
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
(6, 4)
```

Note: `line.inclusive()` is semantically equivalent to doing `Line::new(start,
end).chain(iter::once(end))` but it does not have any iterator chaining
overhead.
