# bresenham

Iterator-based [Bresenham](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm) rasterizers.

By default this crate ships a 2D line walker (`line`). Other primitives from
[Alois Zingl's notes](https://zingl.github.io/bresenham.html) are optional Cargo
features. It calculates coordinates without knowing anything about drawing
methods or surfaces.

| Feature   | Types                                      | Interval                 |
|-----------|--------------------------------------------|--------------------------|
| `line`    | `Line` (default)                           | half-open `[start, end)` |
| `line3d`  | `Line3d`                                   | half-open `[start, end)` |
| `circle`  | `Circle`                                   | closed outline           |
| `ellipse` | `Ellipse`, `EllipseRect`                   | closed outline           |
| `bezier`  | `QuadBezier`                               | inclusive `[start, end]` |
| `aa`      | `LineAA`, `WideLine`, `QuadBezierAA`       | inclusive `[start, end]` |

Integer lines are half-open: `start` is included, `end` is not. Anti-aliased
lines and quadratic Béziers include both endpoints. Circles and ellipses are
closed outlines. Half-open vs inclusive is a performance choice, not a
convenience wrapper.

`Bresenham`, `Bresenham3d`, and `BresenhamAA` are type aliases for `Line`,
`Line3d`, and `LineAA`.

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
