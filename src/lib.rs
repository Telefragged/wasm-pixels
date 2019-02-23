extern crate js_sys;
extern crate wasm_bindgen;

use std::ops::*;
use wasm_bindgen::prelude::*;

const MAX_DETONATE_TIME: f32 = 10.0;
const MIN_DETONATE_TIME: f32 = 3.0;

const MAX_EVENTS_PER_TICK: usize = 10;

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
enum State {
    Idle,
    Detonating(f32)
}

#[derive(Clone, Copy)]
struct Dot {
    pos: Vec2d,
    dir: Vec2d,
    state: State
}

impl Dot {
    pub fn tick(&self, time_delta: f32, width: f32, height: f32) -> Dot {
        let length = self.dir.length();

        let new_state = match self.state {
            State::Idle => State::Idle,
            State::Detonating(t) => State::Detonating(t - time_delta)
        };

        if length == 0.0 {
            Dot {
                pos: self.pos,
                dir: self.dir,
                state: new_state
            }
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
                state: new_state
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
impl Universe {
    fn handle_events(&mut self, depth: usize) {
        let interpolate = |min: f32, max: f32, factor: f32| min + (max - min) * factor.powi(2);

        match (self.pending_events.pop(), depth < MAX_EVENTS_PER_TICK) {
            (Some(event), true) => {
                self.dots =
                    self.dots
                    .iter()
                    .map(|dot| {
                        if (dot.pos.x - event.position.x).abs() > event.radius && (dot.pos.y - event.position.y).abs() > event.radius {
                            return dot.clone();
                        }

                        let dir = (dot.pos - event.position).normalized();
                        let dist = (dot.pos - event.position).length();

                        if dist >= event.radius {
                            dot.clone()
                        } else {
                            let new_state = match dot.state {
                                State::Idle => State::Detonating( random_f32() * (MAX_DETONATE_TIME - MIN_DETONATE_TIME) + MIN_DETONATE_TIME),
                                State::Detonating(t) => State::Detonating(t)
                            };

                            Dot {
                                pos: dot.pos,
                                dir: dot.dir
                                    + (dir * interpolate(0.0, 50.0, 1.0 - dist / event.radius)),
                                state: new_state
                            }
                        }
                    })
                    .collect();
                self.handle_events(depth + 1)
            }
            _ => ()
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

            let interpolate = |min: f32, max: f32, factor: f32| min + (max - min) * (1.0 - factor).max(0.0).min(1.0);

            if first_index <= self.image_data.len() {
                let red = match dot.state {
                    State::Detonating(t) => interpolate(0.0, 255.0, t / MAX_DETONATE_TIME) as u8,
                    _ => 0
                };

                self.image_data[first_index] = red;

                self.image_data[first_index + 3] = 255;
            }
        }
    }

    pub fn tick(&mut self, time_delta: f32) {

        let only_alive = |dot : &&Dot| {
            match dot.state {
                State::Detonating(t) => t > 0.0,
                _ => true
            }
        };

        let only_dead = |dot : &&Dot| !only_alive(dot);

        let mut new_events : Vec<MouseEvent> = self
            .dots
            .iter()
            .filter(only_dead)
            .map(|dot| MouseEvent {position: dot.pos, radius: 10.0})
            .collect();

        self.pending_events.append(&mut new_events);

        let new_dots: Vec<Dot> = self
            .dots
            .iter()
            .filter(only_alive)
            .map(|dot| dot.tick(time_delta, self.width() as f32, self.height() as f32))
            .collect();

        self.dots = new_dots;

        self.handle_events(0);
    }

    pub fn new(width: u32, height: u32, num_dots: u32) -> Universe {
        let size = (width * height) as usize;

        let mut dots = Vec::with_capacity(size);

        let random_vec = || Vec2d {
            x: random_f32() * width as f32,
            y: random_f32() * height as f32,
        };

        for _ in 0..num_dots {
            let dir = Vec2d {x: 0.0, y: 0.0};

            dots.push(Dot {
                pos: random_vec(),
                dir: dir,
                state: State::Idle
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
