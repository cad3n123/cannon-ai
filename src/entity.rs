#![allow(dead_code)]

use std::f32::consts::PI;

use raylib::{
    color::Color,
    ffi::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

use crate::TWO_PI;

pub const CANNON_RADIUS: f32 = 50.0;
pub const BARREL_HEIGHT: f32 = 40.0;
const BARREL_WIDTH: f32 = 2.0 * BARREL_HEIGHT / 3.0;
const ENEMY_SIZE: usize = 10;
pub const ENEMY_WIDTH: f32 = 7.5 * ENEMY_SIZE as f32;
pub const ENEMY_HEIGHT: f32 = 10.0 * ENEMY_SIZE as f32;
const BULLET_SIZE: usize = 10;
const BULLET_WIDTH: f32 = 1.5 * BULLET_SIZE as f32;
pub const BULLET_HEIGHT: f32 = 2.5 * BULLET_SIZE as f32;

#[derive(Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn sum_to_borrowed(&mut self, other: &Point) -> &mut Point {
        self.x += other.x;
        self.y += other.y;
        self
    }
    pub fn sum_to_owned(mut self, other: &Point) -> Point {
        self.sum_to_borrowed(other);
        self
    }
    pub fn sum(&self, other: &Point) -> Point {
        self.clone().sum_to_owned(other)
    }
    pub fn difference_to_borrowed(&mut self, other: &Point) -> &mut Point {
        self.x -= other.x;
        self.y -= other.y;
        self
    }
    pub fn difference_to_owned(mut self, other: &Point) -> Point {
        self.difference_to_borrowed(other);
        self
    }
    pub fn difference(&self, other: &Point) -> Point {
        self.clone().difference_to_owned(other)
    }
    pub fn scale_to_borrowed(&mut self, scalar: f32) -> &mut Point {
        self.x *= scalar;
        self.y *= scalar;
        self
    }
    pub fn scale_to_owned(mut self, scalar: f32) -> Point {
        self.scale_to_borrowed(scalar);
        self
    }
    pub fn scale(&self, scalar: f32) -> Point {
        self.clone().scale_to_owned(scalar)
    }
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    pub fn arc_tan(&self) -> f32 {
        let mut angle = (self.y / self.x).atan();
        if self.x < 0.0 {
            angle += PI;
        } else if self.y < 0.0 {
            angle += TWO_PI;
        }

        if angle > TWO_PI {
            angle -= TWO_PI;
        }

        angle
    }
}

pub trait Sprite {
    fn position(&self) -> &Point;
    fn position_mut(&mut self) -> &mut Point;
    fn draw(&self, d: &mut RaylibDrawHandle<'_>);
}

pub struct Cannon {
    pub position: Point,
    pub direction: f32,
}

impl Cannon {
    pub fn new() -> Self {
        Self {
            position: Point { x: 400.0, y: 300.0 },
            direction: 0.0,
        }
    }
}

impl Sprite for Cannon {
    fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        const HALF_BARREL_HEIGHT: f32 = BARREL_HEIGHT / 2.0;
        const HALF_BARREL_WIDTH: f32 = BARREL_WIDTH / 2.0;
        d.draw_circle(
            self.position.x as i32,
            self.position.y as i32,
            CANNON_RADIUS,
            Color::BLACK,
        );
        d.draw_rectangle_pro(
            raylib::ffi::Rectangle {
                x: self.position.x
                    + self.direction.cos() * (CANNON_RADIUS + HALF_BARREL_HEIGHT - 5.0),
                y: self.position.y
                    + self.direction.sin() * (CANNON_RADIUS + HALF_BARREL_HEIGHT - 5.0),
                width: BARREL_HEIGHT,
                height: BARREL_WIDTH,
            },
            Vector2 {
                x: HALF_BARREL_HEIGHT,
                y: HALF_BARREL_WIDTH,
            },
            self.direction * 180.0 / PI,
            Color::BLACK,
        );
    }

    fn position(&self) -> &Point {
        &self.position
    }

    fn position_mut(&mut self) -> &mut Point {
        &mut self.position
    }
}

pub trait Entity {
    fn update(&mut self, delta_time: f32);
}

pub struct Bullet {
    pub position: Point,
    pub direction: f32,
    pub velocity: Point,
}

impl Sprite for Bullet {
    fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        d.draw_rectangle_pro(
            raylib::ffi::Rectangle {
                x: self.position.x,
                y: self.position.y,
                width: BULLET_HEIGHT,
                height: BULLET_WIDTH,
            },
            Vector2 {
                x: BULLET_HEIGHT / 2.0,
                y: BULLET_WIDTH / 2.0,
            },
            self.direction * 180.0 / PI,
            Color::BLACK,
        );
    }

    fn position(&self) -> &Point {
        &self.position
    }

    fn position_mut(&mut self) -> &mut Point {
        &mut self.position
    }
}

impl Entity for Bullet {
    fn update(&mut self, delta_time: f32) {
        self.position
            .sum_to_borrowed(&self.velocity.scale(delta_time));
    }
}
pub struct Enemy {
    pub position: Point,
    pub direction: f32,
    pub velocity: Point,
}

impl Sprite for Enemy {
    fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        const HALF_ENEMY_HEIGHT: f32 = ENEMY_HEIGHT / 2.0;
        const HALF_ENEMY_WIDTH: f32 = ENEMY_WIDTH / 2.0;
        let direction_cos = self.direction.cos();
        let direction_sin = self.direction.sin();
        d.draw_triangle(
            Vector2 {
                x: self.position.x + direction_cos * HALF_ENEMY_HEIGHT,
                y: self.position.y + direction_sin * HALF_ENEMY_HEIGHT,
            },
            Vector2 {
                x: self.position.x - direction_cos * HALF_ENEMY_HEIGHT
                    + direction_sin * HALF_ENEMY_WIDTH,
                y: self.position.y
                    - direction_cos * HALF_ENEMY_WIDTH
                    - direction_sin * HALF_ENEMY_HEIGHT,
            },
            Vector2 {
                x: self.position.x
                    - direction_cos * HALF_ENEMY_HEIGHT
                    - direction_sin * HALF_ENEMY_WIDTH,
                y: self.position.y + direction_cos * HALF_ENEMY_WIDTH
                    - direction_sin * HALF_ENEMY_HEIGHT,
            },
            Color::BLACK,
        );
    }

    fn position(&self) -> &Point {
        &self.position
    }

    fn position_mut(&mut self) -> &mut Point {
        &mut self.position
    }
}
impl Entity for Enemy {
    fn update(&mut self, delta_time: f32) {
        self.position
            .sum_to_borrowed(&self.velocity.scale(delta_time));
    }
}
