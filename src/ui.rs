#![allow(dead_code)]

use crate::entity::{Point, Sprite};
use raylib::consts::MouseButton;
use raylib::prelude::*;

#[allow(clippy::type_complexity)]
pub struct Button {
    pub position: Point,
    pub font_color: Color,
    pub text: String,
    pub font_size: i32,
    pub on_mouse_enter: Option<Box<dyn FnMut(&mut Self)>>,
    pub on_mouse_hover: Option<Box<dyn FnMut(&mut Self)>>,
    pub on_mouse_exit: Option<Box<dyn FnMut(&mut Self)>>,
    pub on_mouse_down: Option<Box<dyn FnMut(&mut Self)>>,
    pub on_mouse_up: Option<Box<dyn FnMut(&mut Self)>>,
    mouse_hovering: bool,
}

impl Button {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn build(
        text: String,
        position: &Point,
        font_color: Color,
        on_mouse_enter: Option<Box<dyn FnMut(&mut Self)>>,
        on_mouse_hover: Option<Box<dyn FnMut(&mut Self)>>,
        on_mouse_exit: Option<Box<dyn FnMut(&mut Self)>>,
        on_mouse_down: Option<Box<dyn FnMut(&mut Self)>>,
        on_mouse_up: Option<Box<dyn FnMut(&mut Self)>>,
    ) -> Self {
        Self {
            text,
            position: position.clone(),
            font_color,
            font_size: 24,
            on_mouse_enter,
            on_mouse_hover,
            on_mouse_exit,
            on_mouse_down,
            on_mouse_up,
            mouse_hovering: false,
        }
    }
    pub fn size(&self, d: &RaylibDrawHandle<'_>) -> Vector2 {
        #[allow(clippy::cast_precision_loss)]
        d.get_font_default()
            .measure_text(&self.text, self.font_size as f32, self.spacing())
    }
    #[allow(clippy::cast_precision_loss)]
    pub fn spacing(&self) -> f32 {
        let default_font_size = 10.0_f32;
        self.font_size as f32 / default_font_size
    }
    pub fn update(&mut self, d: &RaylibDrawHandle<'_>) {
        if self.point_inside(d, d.get_mouse_position()) {
            if !self.mouse_hovering {
                self.mouse_hovering = true;
                if let Some(mut callback) = self.on_mouse_enter.take() {
                    callback(self);
                    self.on_mouse_enter = Some(callback);
                }
            }
            if let Some(mut callback) = self.on_mouse_hover.take() {
                callback(self);
                self.on_mouse_hover = Some(callback);
            }
            if d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                if let Some(mut callback) = self.on_mouse_down.take() {
                    callback(self);
                    self.on_mouse_down = Some(callback);
                }
            } else if d.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                if let Some(mut callback) = self.on_mouse_up.take() {
                    callback(self);
                    self.on_mouse_up = Some(callback);
                }
            }
        } else if self.mouse_hovering {
            self.mouse_hovering = false;
            if let Some(mut callback) = self.on_mouse_exit.take() {
                callback(self);
                self.on_mouse_exit = Some(callback);
            }
        }
    }
    fn point_inside(&self, d: &RaylibDrawHandle<'_>, point: Vector2) -> bool {
        let size = self.size(d);

        point.x >= self.position.x
            && point.x <= self.position.x + size.x
            && point.y >= self.position.y
            && point.y <= self.position.y + size.y
    }
}

impl Sprite for Button {
    fn position(&self) -> &Point {
        &self.position
    }
    fn position_mut(&mut self) -> &mut Point {
        &mut self.position
    }
    fn draw(&self, d: &mut RaylibDrawHandle<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        d.draw_text(
            &self.text,
            self.position.x as i32,
            self.position.y as i32,
            self.font_size,
            self.font_color,
        );
    }
}
