use raylib::prelude::RaylibDrawHandle;

pub struct Point {
    pub x: f32,
    pub y: f32
}

pub trait Sprite {
    fn draw(d: &mut RaylibDrawHandle<'_>);
}

pub struct Cannon {
    pub position: Point,
}

impl Cannon {
    pub fn new() -> Self {
        Self {
            position: Point { x: 400.0, y: 300.0 }
        }
    }
}

impl Sprite for Cannon {
    fn draw(d: &mut RaylibDrawHandle<'_>) {
        
    }
}