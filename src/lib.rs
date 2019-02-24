extern crate js_sys;
extern crate wasm_bindgen;

use std::ops::*;
use wasm_bindgen::prelude::*;

const MAX_DETONATE_TIME: f32 = 10.0;
const MIN_DETONATE_TIME: f32 = 3.0;

const EXPLOSION_RADIUS: f32 = 10.0;

const GRID_SIZE: usize = 10;

const MAX_EVENTS_PER_TICK: usize = 50;

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
    Detonating(f32, f32)
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
            State::Detonating(t, max_t) => State::Detonating(t - time_delta, max_t)
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
                pos.x += width - 1.0;
            } else if pos.x >= width {
                pos.x -= width;
            }

            if pos.y < 0.0 {
                pos.y += height - 1.0;
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

struct ForceEvent {
    position: Vec2d,
    radius: f32,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    dots: Vec<Dot>,
    pending_events: Vec<ForceEvent>,
    image_data: Vec<u8>,
    grid_width: usize,
    grid_height: usize,
    grid_columns: usize,
    grid_cells: Vec<Vec<usize>>
}

fn random_f32() -> f32 {
    js_sys::Math::random() as f32
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_usize(a: usize);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_f32_f32(a: f32, b: f32);
}

#[wasm_bindgen]
impl Universe {
    fn get_grid_index(&self, position: Vec2d) -> usize {
        let column = position.x as usize / self.grid_width;
        let row = position.y as usize / self.grid_height;

        column + row * self.grid_columns
    }

    fn get_all_grid_indices(&self, top_left: Vec2d, bottom_right: Vec2d) -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();

        let first_column = top_left.x.max(0.0) as usize / self.grid_width;
        let first_row = top_left.y.max(0.0) as usize / self.grid_height;

        let last_column = bottom_right.x.min((self.width() - 1) as f32) as usize / self.grid_width;
        let last_row = bottom_right.y.min((self.height() - 1) as f32) as usize / self.grid_height;

        for col in first_column..=last_column {
            for row in first_row..=last_row {
                ret.push(col + row * self.grid_columns);
            }
        }

        ret
    }

    fn handle_events(&mut self, max_events: usize) {
        let interpolate = |min: f32, max: f32, factor: f32| min + (max - min) * factor.powi(2);

        for _ in 0..max_events {
            match self.pending_events.pop() {
                Some(event) => {
                    let top_left = Vec2d {x: event.position.x - event.radius, y: event.position.y - event.radius };
                    let bottom_right = Vec2d {x: event.position.x + event.radius, y: event.position.y + event.radius };

                    let grid_indices = self.get_all_grid_indices(top_left, bottom_right);

                    let mut dot_indices: Vec<usize> = grid_indices.iter().map(|index| self.grid_cells[index.clone()].clone()).flatten().collect();

                    for index in dot_indices {
                        let dot = self.dots[index];

                        if (dot.pos.x - event.position.x).abs() > event.radius && (dot.pos.y - event.position.y).abs() > event.radius {
                            continue;
                        }

                        let dir = (dot.pos - event.position).normalized();
                        let dist = (dot.pos - event.position).length();

                        if dist >= event.radius {
                            continue;
                        } else {
                            let new_state = match dot.state {
                                State::Idle => {
                                    let time_until_detonation = random_f32() * (MAX_DETONATE_TIME - MIN_DETONATE_TIME) + MIN_DETONATE_TIME;
                                    State::Detonating( time_until_detonation, time_until_detonation)
                                },
                                State::Detonating(t, max_t) => State::Detonating(t, max_t)
                            };

                            let dot = Dot {
                                pos: dot.pos,
                                dir: dot.dir
                                    + (dir * interpolate(0.0, 50.0, 1.0 - dist / event.radius)),
                                state: new_state
                            };

                            self.dots[index] = dot;
                        }
                    }
                }
                _ => return
            }
        }
    }

    pub fn render_image_data(&mut self) {
        for x in (0..self.image_data.len()).step_by(4) {
            self.image_data[x] = 0;
            self.image_data[x + 3] = 0;
        }

        let width = self.width() as usize;

        let get_red_index = |x: usize, y: usize| (x + (y * width as usize)) * 4;

        for dot in &self.dots {
            let first_index = get_red_index(dot.pos.x as usize, dot.pos.y as usize);

            let interpolate = |min: f32, max: f32, factor: f32| min + (max - min) * (1.0 - factor).max(0.0).min(1.0);

            if first_index <= self.image_data.len() {
                let red = match dot.state {
                    State::Detonating(t, max_t) => interpolate(0.0, 255.0, t / max_t) as u8,
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
                State::Detonating(t, _) => t > 0.0,
                _ => true
            }
        };

        let only_dead = |dot : &&Dot| !only_alive(dot);

        let new_events : Vec<ForceEvent> = self
            .dots
            .iter()
            .filter(only_dead)
            .map(|dot| ForceEvent {position: dot.pos, radius: EXPLOSION_RADIUS})
            .collect();

        self.pending_events.extend(new_events);

        let new_dots: Vec<Dot> = self
            .dots
            .iter()
            .filter(only_alive)
            .map(|dot| dot.tick(time_delta, self.width() as f32, self.height() as f32))
            .collect();

        self.dots = new_dots;

        for grid in self.grid_cells.iter_mut() {
            grid.clear();
        }

        for (i, dot) in self.dots.iter().enumerate() {
            let index = self.get_grid_index(dot.pos);

            self.grid_cells[index].push(i);
        }

        self.handle_events(MAX_EVENTS_PER_TICK);
    }

    pub fn new(width: u32, height: u32, num_dots: usize) -> Universe {
        let grid_columns = width as usize / GRID_SIZE as usize;
        let grid_rows = height as usize / GRID_SIZE as usize;

        let num_grids = grid_columns * grid_rows;

        let grid_width: usize = GRID_SIZE as usize;
        let grid_height: usize = GRID_SIZE as usize;

        let mut grid_cells: Vec<Vec<usize>> = vec![Vec::with_capacity((num_dots as usize / num_grids) * 2); num_grids];

        let mut dots = Vec::with_capacity(num_dots);

        let random_vec = || Vec2d {
            x: random_f32() * width as f32,
            y: random_f32() * height as f32,
        };

        for i in 0..num_dots {
            let pos = random_vec();

            let dir = Vec2d {x: 0.0, y: 0.0};
            dots.push(Dot {
                pos: pos,
                dir: dir,
                state: State::Idle
            });

            let column = pos.x as usize / grid_width;
            let row = pos.y as usize / grid_height;

            let grid_index = column + row * grid_columns;

            grid_cells[grid_index].push(i);
        }

        let pending_events: Vec<ForceEvent> = Vec::new();

        let image_data = vec![0; width as usize * height as usize * 4];

        Universe {
            width,
            height,
            dots,
            pending_events,
            image_data,
            grid_width,
            grid_height,
            grid_columns,
            grid_cells
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn add_event(&mut self, x: f32, y: f32, radius: f32) {
        self.pending_events.push(ForceEvent {
            position: Vec2d { x, y },
            radius,
        })
    }

    pub fn image_data(&self) -> *const u8 {
        self.image_data.as_ptr()
    }

    pub fn remaining_dots(&self) -> usize {
        self.dots.len()
    }
}
