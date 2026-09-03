#![cfg(all(feature = "ellipse-aa", feature = "fill"))]

use std::collections::{BTreeMap, BTreeSet};

use nano9_raster::{EllipseAa, Fill, Plot, Point};

fn outline(a: isize, b: isize) -> BTreeMap<Point, u8> {
    let mut pixels = BTreeMap::new();
    for (point, alpha) in EllipseAa::new((0, 0), a, b) {
        pixels
            .entry(point)
            .and_modify(|old: &mut u8| *old = (*old).max(alpha))
            .or_insert(alpha);
    }
    pixels
}

fn fill(a: isize, b: isize) -> BTreeMap<Point, u8> {
    let mut pixels = BTreeMap::new();
    let mut span_rows = BTreeSet::new();
    for plot in EllipseAa::new((0, 0), a, b).fill() {
        match plot {
            Plot::Span(span) => {
                assert!(span.x0 <= span.x1);
                assert!(span_rows.insert(span.y), "second span on row {}", span.y);
                for x in span.x0..=span.x1 {
                    assert!(pixels.insert((x, span.y), 255).is_none());
                }
            }
            Plot::Point((point, alpha)) => {
                assert!(alpha > 0);
                assert!(pixels.insert(point, alpha).is_none(), "{point:?} twice");
            }
        }
    }
    pixels
}

fn oracle(a: isize, b: isize, samples: isize) -> BTreeMap<Point, u8> {
    let mut pixels = BTreeMap::new();
    let total = samples * samples;
    for y in -b - 1..=b + 1 {
        for x in -a - 1..=a + 1 {
            let mut inside = 0;
            for sy in 0..samples {
                for sx in 0..samples {
                    let px = x as f64 + (sx as f64 + 0.5) / samples as f64 - 0.5;
                    let py = y as f64 + (sy as f64 + 0.5) / samples as f64 - 0.5;
                    if px * px / (a * a) as f64 + py * py / (b * b) as f64 <= 1.0 {
                        inside += 1;
                    }
                }
            }
            let alpha = ((inside * 255 + total / 2) / total) as u8;
            if alpha > 0 {
                pixels.insert((x, y), alpha);
            }
        }
    }
    pixels
}

#[test]
fn outline_tracks_minor_axis_distance() {
    for &(a, b) in &[(1, 4), (2, 7), (5, 2), (7, 3), (12, 5)] {
        let actual = outline(a, b);
        for y in -b - 1..=b + 1 {
            for x in -a - 1..=a + 1 {
                let vertical = b * b * x.abs() <= a * a * y.abs();
                let distance = if vertical {
                    let boundary = b as f64 * (1.0 - (x as f64 / a as f64).powi(2)).max(0.0).sqrt();
                    (y.abs() as f64 - boundary).abs()
                } else {
                    let boundary = a as f64 * (1.0 - (y as f64 / b as f64).powi(2)).max(0.0).sqrt();
                    (x.abs() as f64 - boundary).abs()
                };
                let expected = (255.0 * (1.0 - distance).max(0.0)).round() as u8;
                let got = *actual.get(&(x, y)).unwrap_or(&0);
                assert!(
                    got.abs_diff(expected) <= 55,
                    "a={a} b={b} ({x},{y}) got={got} expected={expected}"
                );
            }
        }
    }
}

#[test]
fn fill_tracks_supersampled_area() {
    for &(a, b) in &[(1, 4), (2, 7), (5, 2), (7, 3), (12, 5), (5, 12)] {
        let actual = fill(a, b);
        let expected = oracle(a, b, 24);
        let keys: BTreeSet<_> = actual.keys().chain(expected.keys()).copied().collect();
        let mut sum = 0u64;
        let mut max = 0u8;
        let mut worst = ((0, 0), 0, 0);
        for point in &keys {
            let got = *actual.get(point).unwrap_or(&0);
            let want = *expected.get(point).unwrap_or(&0);
            let error = got.abs_diff(want);
            sum += u64::from(error);
            if error > max {
                max = error;
                worst = (*point, got, want);
            }
        }
        let mean = sum as f64 / keys.len() as f64;
        assert!(mean <= 18.0, "a={a} b={b} mean={mean:.1} max={max}");
        assert!(
            max <= 80,
            "a={a} b={b} max={max} at {:?}: {} vs {}",
            worst.0,
            worst.1,
            worst.2
        );
    }
}
