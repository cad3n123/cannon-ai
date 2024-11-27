use std::f32::consts::PI;

use raylib::{
    color::Color,
    ffi::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

const CANNON_RADIUS: f32 = 50.0;
const BARREL_LENGTH: f32 = 40.0;
const BARREL_HEIGHT: f32 = 2.0 * BARREL_LENGTH / 3.0;
const ENEMY_SIZE: usize = 10;
const ENEMY_WIDTH: f32 = 1.4 * ENEMY_SIZE as f32;
const ENEMY_HEIGHT: f32 = 2.0 * ENEMY_SIZE as f32;
const BULLET_SIZE: usize = 10;
const BULLET_WIDTH: f32 = 1.5 * BULLET_SIZE as f32;
const BULLET_HEIGHT: f32 = 2.5 * BULLET_SIZE as f32;

#[derive(Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
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
        d.draw_circle(
            self.position.x as i32,
            self.position.y as i32,
            CANNON_RADIUS,
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

pub struct Bullet {
    pub position: Point,
    pub direction: f32,
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

pub struct Enemy {
    position: Point,
    direction: f32,
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
                x: self.position.x + direction_cos * (-HALF_ENEMY_HEIGHT - HALF_ENEMY_WIDTH),
                y: self.position.y + direction_sin * (-HALF_ENEMY_HEIGHT - HALF_ENEMY_WIDTH),
            },
            Vector2 {
                x: self.position.x + direction_cos * (-HALF_ENEMY_HEIGHT - HALF_ENEMY_WIDTH),
                y: self.position.y + direction_sin * (-HALF_ENEMY_HEIGHT + HALF_ENEMY_WIDTH),
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
