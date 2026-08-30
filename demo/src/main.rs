//! Pixelated canvas demo for `bresenham`. Autoplay cycles primitives; drag to draw.

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
const REVEAL_FRAMES: usize = 45;
const HOLD_FRAMES: u32 = 60;
const IDLE_RESUME_FRAMES: u32 = 180;

/// 3×5 digits 0–9, bit 0 = top-left, row-major.
const DIGITS: [u16; 10] = [
    0b111_101_101_101_111,
    0b010_110_010_010_111,
    0b111_001_111_100_111,
    0b111_001_111_001_111,
    0b101_101_111_001_001,
    0b111_100_111_001_111,
    0b111_100_111_101_111,
    0b111_001_001_001_001,
    0b111_101_111_101_111,
    0b111_101_111_001_111,
];

struct Demo {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    buf: Vec<u8>,
    start: Point,
    end: Point,
    dragging: bool,
    kind: Kind,
    pixels: Vec<(Point, u8)>,
    tour: u32,
    mode: Mode,
}

enum Mode {
    Auto { step: usize, hold: u32 },
    Click { idle: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    fn index(self) -> usize {
        match self {
            Kind::Line => 0,
            Kind::LineAA => 1,
            Kind::Circle => 2,
            Kind::EllipseRect => 3,
            Kind::QuadBezier => 4,
            Kind::QuadBezierAA => 5,
            Kind::WideLine => 6,
            Kind::FillCircle => 7,
        }
    }

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
            dragging: false,
            kind: Kind::Line,
            pixels: Vec::new(),
            tour: 0,
            mode: Mode::Auto { step: 0, hold: 0 },
        };
        demo.load_shape();
        demo.clear();
        demo.present()?;
        Ok(demo)
    }

    fn geometry(kind: Kind, tour: u32) -> (Point, Point) {
        let i = (tour as usize) % 3;
        match kind {
            Kind::Line => [
                ((8, 14), (58, 38)),
                ((8, 38), (58, 12)),
                ((12, 24), (56, 24)),
            ][i],
            Kind::LineAA => [
                ((8, 36), (58, 14)),
                ((10, 12), (54, 40)),
                ((12, 32), (56, 16)),
            ][i],
            Kind::Circle => [
                ((32, 26), (46, 26)),
                ((28, 28), (40, 28)),
                ((36, 24), (48, 24)),
            ][i],
            Kind::EllipseRect => [
                ((14, 12), (54, 40)),
                ((18, 16), (50, 36)),
                ((10, 20), (58, 32)),
            ][i],
            Kind::QuadBezier => [
                ((8, 36), (58, 36)),
                ((10, 12), (54, 40)),
                ((8, 40), (58, 14)),
            ][i],
            Kind::QuadBezierAA => [
                ((8, 14), (58, 14)),
                ((10, 40), (54, 12)),
                ((8, 38), (58, 20)),
            ][i],
            Kind::WideLine => [
                ((10, 16), (54, 36)),
                ((10, 36), (54, 14)),
                ((12, 22), (56, 28)),
            ][i],
            Kind::FillCircle => [
                ((32, 26), (42, 26)),
                ((28, 28), (37, 28)),
                ((36, 24), (45, 24)),
            ][i],
        }
    }

    fn load_shape(&mut self) {
        let (start, end) = Self::geometry(self.kind, self.tour);
        self.start = start;
        self.end = end;
        self.pixels = self.collect_pixels();
    }

    fn collect_pixels(&self) -> Vec<(Point, u8)> {
        let start = self.start;
        let end = self.end;
        match self.kind {
            Kind::Line => Line::new(start, end).map(|p| (p, 255)).collect(),
            Kind::LineAA => LineAA::new(start, end).filter(|(_, c)| *c > 0).collect(),
            Kind::Circle => Circle::new(start, Self::radius(start, end))
                .map(|p| (p, 255))
                .collect(),
            Kind::EllipseRect => EllipseRect::new(start, end).map(|p| (p, 255)).collect(),
            Kind::QuadBezier => {
                let c = Self::control_point(start, end);
                QuadBezier::new(start, c, end).map(|p| (p, 255)).collect()
            }
            Kind::QuadBezierAA => {
                let c = Self::control_point(start, end);
                QuadBezierAA::new(start, c, end)
                    .filter(|(_, c)| *c > 0)
                    .collect()
            }
            Kind::WideLine => WideLine::new(start, end, 3.0)
                .filter(|(_, c)| *c > 0)
                .collect(),
            Kind::FillCircle => Circle::new(start, Self::radius(start, end))
                .fill()
                .flat_map(|h| (h.x0..=h.x1).map(move |x| ((x, h.y), 255)))
                .collect(),
        }
    }

    fn control_point(start: Point, end: Point) -> Point {
        let (x0, y0) = start;
        let (x1, y1) = end;
        ((x0 + x1) / 2 - (y1 - y0) / 3, (y0 + y1) / 2 + (x1 - x0) / 3)
    }

    fn radius(start: Point, end: Point) -> isize {
        (end.0 - start.0).abs().max((end.1 - start.1).abs()).max(1)
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

    fn stamp_digit(&mut self) {
        for y in 0..7 {
            for x in 0..5 {
                self.plot((x, y), BG[0], BG[1], BG[2]);
            }
        }
        let bits = DIGITS[self.kind.index()];
        for row in 0..5 {
            for col in 0..3 {
                let bit = row * 3 + col;
                if (bits >> (14 - bit)) & 1 == 1 {
                    self.plot((1 + col, 1 + row), 255, 255, 255);
                }
            }
        }
    }

    fn present(&mut self) -> Result<(), JsValue> {
        self.stamp_digit();
        let image =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut self.buf), WIDTH, HEIGHT)?;
        self.ctx.put_image_data(&image, 0.0, 0.0)
    }

    fn reveal_count(&self) -> usize {
        let n = self.pixels.len();
        if n == 0 {
            1
        } else {
            n.div_ceil(REVEAL_FRAMES)
        }
    }

    fn advance(&mut self) {
        let next = self.kind.next();
        if next == Kind::Line {
            self.tour = self.tour.wrapping_add(1);
        }
        self.kind = next;
        self.load_shape();
        self.mode = Mode::Auto { step: 0, hold: 0 };
        self.clear();
    }

    fn begin_auto(&mut self) -> Result<(), JsValue> {
        self.dragging = false;
        self.load_shape();
        self.mode = Mode::Auto { step: 0, hold: 0 };
        self.clear();
        self.present()
    }

    fn paint_shape(&mut self) -> Result<(), JsValue> {
        self.pixels = self.collect_pixels();
        self.clear();
        for i in 0..self.pixels.len() {
            let (p, c) = self.pixels[i];
            self.plot(p, c, c, c);
        }
        self.present()
    }

    fn tick(&mut self) -> Result<(), JsValue> {
        match self.mode {
            Mode::Auto { step, hold } => {
                if hold > 0 {
                    if hold == 1 {
                        self.advance();
                    } else {
                        self.mode = Mode::Auto {
                            step,
                            hold: hold - 1,
                        };
                    }
                    return Ok(());
                }
                if self.pixels.is_empty() {
                    self.mode = Mode::Auto {
                        step,
                        hold: HOLD_FRAMES,
                    };
                    return self.present();
                }
                if step == 0 {
                    self.clear();
                }
                let n = self.reveal_count();
                let end = (step + n).min(self.pixels.len());
                for i in step..end {
                    let (p, c) = self.pixels[i];
                    self.plot(p, c, c, c);
                }
                let hold = if end >= self.pixels.len() {
                    HOLD_FRAMES
                } else {
                    0
                };
                self.mode = Mode::Auto { step: end, hold };
                self.present()
            }
            Mode::Click { idle } => {
                if !self.dragging {
                    let idle = idle + 1;
                    if idle >= IDLE_RESUME_FRAMES {
                        self.begin_auto()?;
                    } else {
                        self.mode = Mode::Click { idle };
                    }
                }
                Ok(())
            }
        }
    }

    fn grid_xy(&self, event: &MouseEvent) -> Point {
        let rect = self.canvas.get_bounding_client_rect();
        let w = rect.width().max(1.0);
        let h = rect.height().max(1.0);
        let x =
            ((f64::from(event.client_x()) - rect.left()) * f64::from(WIDTH) / w).floor() as isize;
        let y =
            ((f64::from(event.client_y()) - rect.top()) * f64::from(HEIGHT) / h).floor() as isize;
        (
            x.clamp(0, WIDTH as isize - 1),
            y.clamp(0, HEIGHT as isize - 1),
        )
    }

    fn on_down(&mut self, event: &MouseEvent) -> Result<(), JsValue> {
        if event.button() == 2 {
            event.prevent_default();
            return self.on_right();
        }
        if event.button() != 0 {
            return Ok(());
        }
        self.mode = Mode::Click { idle: 0 };
        self.dragging = true;
        let p = self.grid_xy(event);
        self.start = p;
        self.end = p;
        self.paint_shape()
    }

    fn on_move(&mut self, event: &MouseEvent) -> Result<(), JsValue> {
        if !self.dragging {
            return Ok(());
        }
        if event.buttons() & 1 == 0 {
            return self.finish_drag(event);
        }
        let p = self.grid_xy(event);
        if p == self.end {
            return Ok(());
        }
        self.end = p;
        self.paint_shape()
    }

    fn on_up(&mut self, event: &MouseEvent) -> Result<(), JsValue> {
        if event.button() != 0 {
            return Ok(());
        }
        self.finish_drag(event)
    }

    fn finish_drag(&mut self, event: &MouseEvent) -> Result<(), JsValue> {
        if !self.dragging {
            return Ok(());
        }
        self.dragging = false;
        self.end = self.grid_xy(event);
        self.mode = Mode::Click { idle: 0 };
        self.paint_shape()
    }

    fn on_right(&mut self) -> Result<(), JsValue> {
        if self.dragging {
            return Ok(());
        }
        let next = self.kind.next();
        if next == Kind::Line {
            self.tour = self.tour.wrapping_add(1);
        }
        self.kind = next;
        match self.mode {
            Mode::Auto { .. } => {
                self.load_shape();
                self.mode = Mode::Auto { step: 0, hold: 0 };
                self.clear();
                self.present()
            }
            Mode::Click { .. } => {
                self.mode = Mode::Click { idle: 0 };
                self.paint_shape()
            }
        }
    }
}

fn request_frame(cb: &Closure<dyn FnMut()>) {
    let _ = web_sys::window()
        .unwrap()
        .request_animation_frame(cb.as_ref().unchecked_ref());
}

fn main() {
    start();
}

fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("grid")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let demo = Rc::new(RefCell::new(Demo::new(canvas.clone()).unwrap()));

    let down_demo = Rc::clone(&demo);
    let on_down = Closure::wrap(Box::new(move |event: MouseEvent| {
        let _ = down_demo.borrow_mut().on_down(&event);
    }) as Box<dyn FnMut(_)>);
    canvas.set_onmousedown(Some(on_down.as_ref().unchecked_ref()));
    on_down.forget();

    let move_demo = Rc::clone(&demo);
    let on_move = Closure::wrap(Box::new(move |event: MouseEvent| {
        let _ = move_demo.borrow_mut().on_move(&event);
    }) as Box<dyn FnMut(_)>);
    canvas.set_onmousemove(Some(on_move.as_ref().unchecked_ref()));
    on_move.forget();

    let up_demo = Rc::clone(&demo);
    let on_up = Closure::wrap(Box::new(move |event: MouseEvent| {
        let _ = up_demo.borrow_mut().on_up(&event);
    }) as Box<dyn FnMut(_)>);
    canvas.set_onmouseup(Some(on_up.as_ref().unchecked_ref()));
    on_up.forget();

    let leave_demo = Rc::clone(&demo);
    let on_leave = Closure::wrap(Box::new(move |event: MouseEvent| {
        let _ = leave_demo.borrow_mut().finish_drag(&event);
    }) as Box<dyn FnMut(_)>);
    canvas.set_onmouseleave(Some(on_leave.as_ref().unchecked_ref()));
    on_leave.forget();

    let on_menu = Closure::wrap(Box::new(move |event: MouseEvent| {
        event.prevent_default();
    }) as Box<dyn FnMut(_)>);
    canvas.set_oncontextmenu(Some(on_menu.as_ref().unchecked_ref()));
    on_menu.forget();

    let raf: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let raf_cb = raf.clone();
    let raf_demo = Rc::clone(&demo);
    *raf.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let _ = raf_demo.borrow_mut().tick();
        if let Some(cb) = raf_cb.borrow().as_ref() {
            request_frame(cb);
        }
    }) as Box<dyn FnMut()>));
    request_frame(raf.borrow().as_ref().unwrap());
}
