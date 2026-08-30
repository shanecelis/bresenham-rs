//! Pixelated canvas demo for `bresenham`. Click two grid points to draw.

use bresenham::{
    Circle, EllipseRect, Fillable, Line, LineAA, Point, QuadBezier, QuadBezierAA, WideLine,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, MouseEvent};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const BG: [u8; 4] = [0x11, 0x11, 0x11, 0xff];

struct Demo {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    buf: Vec<u8>,
    start: Point,
    end: Point,
    pending: Option<Point>,
    kind: Kind,
}

#[derive(Clone, Copy)]
enum Kind {
    Line,
    LineAA,
    Circle,
    EllipseRect,
    QuadBezier,
    QuadBezierAA,
    WideLine,
    FillCircle,
}

impl Kind {
    fn next(self) -> Self {
        match self {
            Kind::Line => Kind::LineAA,
            Kind::LineAA => Kind::Circle,
            Kind::Circle => Kind::EllipseRect,
            Kind::EllipseRect => Kind::QuadBezier,
            Kind::QuadBezier => Kind::QuadBezierAA,
            Kind::QuadBezierAA => Kind::WideLine,
            Kind::WideLine => Kind::FillCircle,
            Kind::FillCircle => Kind::Line,
        }
    }
}

impl Demo {
    fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("no 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;
        ctx.set_image_smoothing_enabled(false);

        let mut demo = Demo {
            ctx,
            canvas,
            buf: vec![0; (WIDTH * HEIGHT * 4) as usize],
            start: (4, 8),
            end: (59, 39),
            pending: None,
            kind: Kind::Line,
        };
        demo.redraw()?;
        Ok(demo)
    }

    fn clear(&mut self) {
        for px in self.buf.chunks_exact_mut(4) {
            px.copy_from_slice(&BG);
        }
    }

    fn plot(&mut self, (x, y): Point, r: u8, g: u8, b: u8) {
        if x < 0 || y < 0 || x >= WIDTH as isize || y >= HEIGHT as isize {
            return;
        }
        let i = ((y as u32 * WIDTH + x as u32) * 4) as usize;
        self.buf[i] = r;
        self.buf[i + 1] = g;
        self.buf[i + 2] = b;
        self.buf[i + 3] = 255;
    }

    fn plot_on(&mut self, p: Point) {
        self.plot(p, 255, 255, 255);
    }

    fn plot_aa(&mut self, (p, cover): (Point, u8)) {
        if cover == 0 {
            return;
        }
        self.plot(p, cover, cover, cover);
    }

    fn plot_hline(&mut self, h: bresenham::HLine) {
        for x in h.x0..=h.x1 {
            self.plot_on((x, h.y));
        }
    }

    fn control_point(start: Point, end: Point) -> Point {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let mx = (x0 + x1) / 2;
        let my = (y0 + y1) / 2;
        let dx = x1 - x0;
        let dy = y1 - y0;
        (mx - dy / 3, my + dx / 3)
    }

    fn radius(start: Point, end: Point) -> isize {
        let dx = (end.0 - start.0).abs();
        let dy = (end.1 - start.1).abs();
        dx.max(dy).max(1)
    }

    fn stamp(&mut self) {
        let start = self.start;
        let end = self.end;
        match self.kind {
            Kind::Line => {
                for p in Line::new(start, end) {
                    self.plot_on(p);
                }
            }
            Kind::LineAA => {
                for px in LineAA::new(start, end) {
                    self.plot_aa(px);
                }
            }
            Kind::Circle => {
                for p in Circle::new(start, Self::radius(start, end)) {
                    self.plot_on(p);
                }
            }
            Kind::EllipseRect => {
                for p in EllipseRect::new(start, end) {
                    self.plot_on(p);
                }
            }
            Kind::QuadBezier => {
                let c = Self::control_point(start, end);
                for p in QuadBezier::new(start, c, end) {
                    self.plot_on(p);
                }
            }
            Kind::QuadBezierAA => {
                let c = Self::control_point(start, end);
                for px in QuadBezierAA::new(start, c, end) {
                    self.plot_aa(px);
                }
            }
            Kind::WideLine => {
                for px in WideLine::new(start, end, 3.0) {
                    self.plot_aa(px);
                }
            }
            Kind::FillCircle => {
                for h in Circle::new(start, Self::radius(start, end)).fill() {
                    self.plot_hline(h);
                }
            }
        }
    }

    fn blit(&mut self) -> Result<(), JsValue> {
        let image =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut self.buf), WIDTH, HEIGHT)?;
        self.ctx.put_image_data(&image, 0.0, 0.0)
    }

    fn redraw(&mut self) -> Result<(), JsValue> {
        self.clear();
        self.stamp();
        self.blit()
    }

    fn grid_xy(&self, event: &MouseEvent) -> Point {
        let cw = self.canvas.client_width().max(1) as f64;
        let ch = self.canvas.client_height().max(1) as f64;
        let x = (event.offset_x() as f64 * f64::from(WIDTH) / cw).floor() as isize;
        let y = (event.offset_y() as f64 * f64::from(HEIGHT) / ch).floor() as isize;
        (
            x.clamp(0, WIDTH as isize - 1),
            y.clamp(0, HEIGHT as isize - 1),
        )
    }

    fn on_click(&mut self, event: &MouseEvent) -> Result<(), JsValue> {
        let p = self.grid_xy(event);
        match self.pending {
            None => {
                self.pending = Some(p);
                self.clear();
                self.plot(p, 0x88, 0xcc, 0xff);
                self.blit()
            }
            Some(start) => {
                self.start = start;
                self.end = p;
                self.pending = None;
                self.redraw()?;
                self.kind = self.kind.next();
                Ok(())
            }
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        web_sys::console::error_1(&info.to_string().into());
    }));
    if let Err(e) = start() {
        web_sys::console::error_1(&e);
    }
}

fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id("grid")
        .ok_or_else(|| JsValue::from_str("missing #grid"))?
        .dyn_into::<HtmlCanvasElement>()?;

    let demo = Rc::new(RefCell::new(Demo::new(canvas.clone())?));
    let click_demo = Rc::clone(&demo);
    let on_down = Closure::wrap(Box::new(move |event: MouseEvent| {
        if let Err(e) = click_demo.borrow_mut().on_click(&event) {
            web_sys::console::error_1(&e);
        }
    }) as Box<dyn FnMut(_)>);
    canvas.set_onmousedown(Some(on_down.as_ref().unchecked_ref()));
    on_down.forget();
    Ok(())
}
