extern crate wasm_bindgen;
extern crate js_sys;

use wasm_bindgen::prelude::*;
use std::ops::*;

#[derive(Copy, Clone)]
struct Vec2d {
    x: f32,
    y: f32
}

impl Vec2d {
    pub fn length(&self) -> f32 {
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

impl Sub for Vec2d {
    type Output = Vec2d;

    fn sub(self, other: Vec2d) -> Vec2d {
        Vec2d { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<f32> for Vec2d {
    type Output = Vec2d;

    fn mul(self, scalar: f32) -> Vec2d {
        Vec2d {x: self.x * scalar, y: self.y * scalar}
    }
}

#[derive(Clone, Copy)]
pub struct Dot {
    pos: Vec2d,
    dir: Vec2d
}

impl Dot {
    pub fn tick(&self, time_delta: f32, width: f32, height: f32) -> Dot {
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

        let friction_coefficient = 0.025;
        let constant_friction = 0.005;

        let length = self.dir.length();

        let desired_length = length * (1.0 - friction_coefficient * time_delta) - constant_friction;

        let new_dir = if desired_length > 0.0 {
            self.dir * (desired_length / length)
        } else {
            Vec2d {x: 0.0, y: 0.0}
        };

        Dot {pos: pos, dir: new_dir}
    }
}

struct MouseEvent {
    position: Vec2d,
    radius: f32
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    dots: Vec<Dot>,
    pending_events: Vec<MouseEvent>
}

fn random_f32() -> f32 {
    js_sys::Math::random() as f32
}

#[wasm_bindgen]
impl Universe {
    fn handle_events(&mut self) {
        let interpolate = |min: f32, max: f32, factor: f32| (min) + (max - min) * factor.powi(3);

        match self.pending_events.pop() {   
            Some(event) => {
                self.dots = self.dots.iter().map(|dot| {
                    let dir = (dot.pos - event.position).normalized();
                    let dist = (dot.pos - event.position).length();
                    if dist >= event.radius {
                        dot.clone()
                    } else {
                        Dot {pos: dot.pos, dir: dot.dir + (dir * interpolate(0.0, 5.0, 1.0 - dist / event.radius))}
                    }
                    }).collect();
                self.handle_events()
            },
            None => ()
        }
    }

    pub fn tick(&mut self, time_delta: f32) {
        self.handle_events();

        let new_dots : Vec<Dot> = self.dots.iter().map(|dot| dot.tick(time_delta, self.width() as f32, self.height() as f32)).collect();

        self.dots = new_dots;
    }

    pub fn new(width: u32, height: u32, num_dots: u32) -> Universe {

        let size = (width * height) as usize;

        let mut dots = Vec::with_capacity(size);

        let random_vec = || {
            Vec2d{x: random_f32() * width as f32, y: random_f32() * height as f32 }
        };

        for _ in 0..num_dots {
            let dir = (random_vec() * (random_f32() * 2.0 - 1.0)).normalized() * random_f32() * 10.0;

            dots.push(Dot{pos: random_vec(), dir: dir});
        }

        let pending_events = Vec::new();

        Universe {
            width,
            height,
            dots,
            pending_events
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn add_event(& mut self, x: f32, y: f32, radius: f32)
    {
        self.pending_events.push(MouseEvent {position: Vec2d {x, y}, radius})
    }

    pub fn dots(&self) -> *const Dot {
        self.dots.as_ptr()
    }
}
