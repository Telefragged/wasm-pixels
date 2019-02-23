extern crate js_sys;
extern crate wasm_bindgen;

use std::ops::*;
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone)]
struct Vec2d {
    x: f32,
    y: f32,
}

impl Vec2d {
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn with_len(&self, desired_length: f32) -> Vec2d {
        self.clone() * (desired_length / self.length())
    }

    pub fn normalized(&self) -> Vec2d {
        let length = self.length();
        Vec2d {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl Add for Vec2d {
    type Output = Vec2d;

    fn add(self, other: Vec2d) -> Vec2d {
        Vec2d {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2d {
    type Output = Vec2d;

    fn sub(self, other: Vec2d) -> Vec2d {
        Vec2d {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2d {
    type Output = Vec2d;

    fn mul(self, scalar: f32) -> Vec2d {
        Vec2d {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Dot {
    pos: Vec2d,
    dir: Vec2d,
}

impl Dot {
    pub fn tick(&self, time_delta: f32, width: f32, height: f32) -> Dot {
        let length = self.dir.length();

        if length == 0.0 {
            self.clone()
        } else {
            let mut pos = self.pos + (self.dir * time_delta);

            if pos.x < 0.0 {
                pos.x += width;
            } else if pos.x >= width {
                pos.x -= width;
            }

            if pos.y < 0.0 {
                pos.y += height;
            } else if pos.y >= height {
                pos.y -= height;
            }

            let friction_coefficient = 0.35;
            let constant_friction = 0.1;

            // let wind = (Vec2d { x: 1.0, y: 0.3 }).with_len(3.0) * time_delta;

            let desired_length = length * (1.0 - friction_coefficient * time_delta)
                - (constant_friction * time_delta);

            let new_dir = if desired_length > 0.0 {
                self.dir.with_len(desired_length)
            } else {
                Vec2d { x: 0.0, y: 0.0 }
            };

            Dot {
                pos: pos,
                dir: new_dir,
            }
        }
    }
}

struct MouseEvent {
    position: Vec2d,
    radius: f32,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    dots: Vec<Dot>,
    pending_events: Vec<MouseEvent>,
    image_data: Vec<u8>,
}

fn random_f32() -> f32 {
    js_sys::Math::random() as f32
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
impl Universe {
    fn handle_events(&mut self) {
        let interpolate = |min: f32, max: f32, factor: f32| (min) + (max - min) * factor.powi(2);

        match self.pending_events.pop() {
            Some(event) => {
                self.dots =
                    self.dots
                    .iter()
                    .map(|dot| {
                        let dir = (dot.pos - event.position).normalized();
                        let dist = (dot.pos - event.position).length();
                        if dist >= event.radius {
                            dot.clone()
                        } else {
                            Dot {
                                pos: dot.pos,
                                dir: dot.dir
                                    + (dir * interpolate(0.0, 50.0, 1.0 - dist / event.radius)),
                            }
                        }
                    })
                    .collect();
                self.handle_events()
            }
            None => (),
        }
    }

    pub fn render_image_data(&mut self) {
        for x in (0..self.image_data.len()).step_by(4) {
            self.image_data[x + 3] = 0;
        }

        let width = self.width() as usize;

        let get_red_index = |x: usize, y: usize| (x + (y * width as usize)) * 4;

        for dot in &self.dots {
            let first_index = get_red_index(dot.pos.x as usize, dot.pos.y as usize);

            if first_index > self.image_data.len() {
                log(&format!("{}, {}", dot.pos.x, dot.pos.y));
            } else {
                self.image_data[first_index + 3] = 255;
            }
        }
    }

    pub fn tick(&mut self, time_delta: f32) {
        self.handle_events();

        let new_dots: Vec<Dot> = self
            .dots
            .iter()
            .map(|dot| dot.tick(time_delta, self.width() as f32, self.height() as f32))
            .collect();

        self.dots = new_dots;
    }

    pub fn new(width: u32, height: u32, num_dots: u32) -> Universe {
        let size = (width * height) as usize;

        let mut dots = Vec::with_capacity(size);

        let random_vec = || Vec2d {
            x: random_f32() * width as f32,
            y: random_f32() * height as f32,
        };

        for _ in 0..num_dots {
            let dir =
                (random_vec() * (random_f32() * 2.0 - 1.0)).normalized() * random_f32() * 10.0;

            dots.push(Dot {
                pos: random_vec(),
                dir: dir,
            });
        }

        let pending_events: Vec<MouseEvent> = Vec::new();

        let image_data = vec![0; width as usize * height as usize * 4];

        Universe {
            width,
            height,
            dots,
            pending_events,
            image_data,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn add_event(&mut self, x: f32, y: f32, radius: f32) {
        self.pending_events.push(MouseEvent {
            position: Vec2d { x, y },
            radius,
        })
    }

    pub fn image_data(&self) -> *const u8 {
        self.image_data.as_ptr()
    }
}
