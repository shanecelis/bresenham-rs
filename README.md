# bresenham
<img align="right" width="256" height="192" alt="demo" src="https://github.com/user-attachments/assets/91c54fe3-9560-49fa-b77e-536f5d6dbffd" />

Iterator-based
[Bresenham](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm)'s line,
circle, and more drawing algorithms.

Bresenham's line drawing algorithm is a fast algorithm to draw a line between
two points, without any overdraw. This crate implements the fast integer
variant, using an iterator-based approach for flexibility. Most, if not all,
overhead should evaporate when inlined by the compiler. It calculates
coordinates without knowing anything about drawing methods or surfaces.



## Example

```rust
for (x, y) in bresenham::Line::new((0, 1), (6, 4)) {
    println!("({}, {})", x, y);
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

## Shapes

[Alois Zingl's notes](https://zingl.github.io/bresenham.html) on Bresenham were
used to add other shapes, which are available as optional Cargo features.

| Demo | Shape              | Type                         | Interval                 | Feature          |
|------|--------------------|------------------------------|--------------------------|------------------|
| 0    | Line               | `Line`                       | half-open `[start, end)` | `line` (default) |
| 1    | Anti-aliased line  | `LineAa`                     | inclusive `[start, end]` | `aa`             |
| 2    | Circle             | `Circle`                     | closed outline           | `circle`         |
| 3    | Ellipse            | `EllipseRect`                | closed outline           | `ellipse`        |
| 4    | Quadratic Bézier   | `QuadBezier`                 | inclusive `[start, end]` | `bezier`         |
| 5    | Anti-aliased Bézier| `QuadBezierAa`               | inclusive `[start, end]` | `aa`             |
| 6    | Wide line          | `WideLineAa`                 | inclusive `[start, end]` | `aa`             |
| 7    | Filled circle      | `Circle` + `Fill`            | scanlines                | `circle`, `fill` |
| 8    | 3D Line            | `Line3d`                     | half-open `[start, end)` | `line3d`         |
| 9    | Ellipse            | `Ellipse`                    | closed outline           | `ellipse`        |


## Demo

The WASM demo shows a 64×48 canvas that autoplays the 0-7 shapes above. `Line3d`
and center-and-radii `Ellipse` are not shown in the demo.

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
and one of the authors maintains that performance degradation should not be made
convenient.

## Fill

The `fill` Cargo feature adds the `Fill` trait on `Circle`, `Ellipse`, and
`EllipseRect`. It yields one `Span` per row to make rasterizing fill shapes more
efficient. Enable it together with `circle` or `ellipse`.

## Inclusive

The `inclusive` Cargo feature adds the `Inclusive` trait on `Line` and `Line3d`,
which returns an iterator that includes the end point making the interval
inclusive `[start, end]` instead of half-open. This approach
follows [indubitablement2's
work](https://github.com/indubitablement2/bresenham-rs), which is careful not
to incur any runtime penalty.
 
### Inclusive Example

This code will produce the same points as the first example plus `(6, 4)` at the
end.

```rust
for (x, y) in bresenham::Line::new((0, 1), (6, 4)).inclusive() {
    println!("({}, {})", x, y);
}
```

Note: `line.inclusive()` is semantically equivalent to doing `Line::new(start,
end).chain(iter::once(end))` but it does not have any iterator chaining
overhead.

## Bresenham Line Variant Notes

By default lines are drawn on a half-open interval `[start, end)`: the `start`
point is included, but the `end` point is not. This allows one to chain multiple
lines together without any overdraw. However, one can opt-in to the "inclusive"
Cargo feature to get an `line.inclusive()` iterator.

This particular implementation of Bresenham breaks ties in quadrants such that
an inclusive line drawn from `(A, B)` and from `(B, A)` will cover the same
points. Thus lines are now symmetrical.

## Why No Generics?

Other Bresenham implementations use a generic type, and there are [forks of this
project](https://github.com/nsmryan/bresenham-rs/commit/4ccd83759d92bd243f48b52be6826df252a88578)
that switch to `i32` instead of the fixed `isize`. As an experiment, the `Line`
was changed to be generic and benchmarked against different signed types.

| Type    | half-open (horizontal) | vs `isize`   |
|---------|------------------------|--------------|
| `i8`    | 1.07 µs                | 1.40× slower |
| `i16`   | 1.07 µs                | 1.40× slower |
| `i32`   | 780 ns                 | 1.02× slower |
| `i64`   | 766 ns                 | same         |
| `isize` | 766 ns                 | —            |
| `i128`  | 1.32 µs                | 1.73× slower |

The types `i8` and `i16` losing to `i32` and `i64` is probably due to LLVM's
extra extend and truncate. Unfortunately, smaller is not faster, but bigger is
definitely slower as with `i128`. Finally `isize` is the machine-size word, so
one can expect it will be the fastest. Based on this experiment, adding generics
was rejected.

## License

This crate is licensed under the MIT License.
