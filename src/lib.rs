extern crate wasm_bindgen;
extern crate js_sys;

use wasm_bindgen::prelude::*;
use std::ops::*;

#[derive(Copy, Clone)]
struct Vec2d {
    x: f64,
    y: f64
}

impl Vec2d {
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalized(&self) -> Vec2d {
        let length = self.length();
        Vec2d {x: self.x / length, y: self.y / length}
    }
}

impl Add for Vec2d {
    type Output = Vec2d;

    fn add(self, other: Vec2d) -> Vec2d {
        Vec2d { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Mul<f64> for Vec2d {
    type Output = Vec2d;

    fn mul(self, scalar: f64) -> Vec2d {
        Vec2d {x: self.x * scalar, y: self.y * scalar}
    }
}

#[derive(Clone, Copy)]
pub struct Dot {
    pos: Vec2d,
    dir: Vec2d
}

impl Dot {
    pub fn tick(&self, time_delta: f64, width: f64, height: f64) -> Dot {
        let mut pos = self.pos + (self.dir * time_delta);

        if self.pos.x < 0.0 {
            pos.x += width;
            pos.y = (pos.y - height / 2.0) * -1.0 + (height / 2.0);
        } else if self.pos.x > width {
            pos.x -= width;
            pos.y = (pos.y - height / 2.0) * -1.0 + (height / 2.0);
        }

        if self.pos.y < 0.0 {
            pos.y += height;
            pos.x = (pos.x - width / 2.0) * -1.0 + (width / 2.0);
        } else if self.pos.y > height {
            pos.y -= height;
            pos.x = (pos.x - width / 2.0) * -1.0 + (width / 2.0);
        }

        Dot {pos: pos, dir: self.dir}
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    dots: Vec<Dot>,
    points: Vec<f64>
}

#[wasm_bindgen]
impl Universe {

    pub fn tick(&mut self, time_delta: f64) {
        let new_dots : Vec<Dot> = self.dots.iter().map(|dot| dot.tick(time_delta, self.width() as f64, self.height() as f64)).collect();

        self.dots = new_dots;

        let new_points : Vec<f64> = self.dots.iter().fold(Vec::with_capacity(self.dots.len() * 2), |mut acc, dot| {acc.push(dot.pos.x); acc.push(dot.pos.y); acc});

        self.points = new_points;
    }

    pub fn new(width: u32, height: u32, num_dots: u32) -> Universe {

        let size = (width * height) as usize;

        let mut dots = Vec::with_capacity(size);

        let random_vec = || {
            Vec2d{x: js_sys::Math::random() * width as f64, y: js_sys::Math::random() * height as f64 }
        };

        for _ in 0..num_dots {
            let dir = (random_vec() * (js_sys::Math::random() * 2.0 - 1.0)).normalized() * js_sys::Math::random() * 5.0;

            dots.push(Dot{pos: random_vec(), dir: dir});
        }

        let points : Vec<f64> = dots.iter().fold(Vec::with_capacity(dots.len() * 2), |mut acc, dot| {acc.push(dot.pos.x); acc.push(dot.pos.y); acc});

        Universe {
            width,
            height,
            dots,
            points
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn dots(&self) -> *const f64 {
        self.points.as_ptr()
    }
}