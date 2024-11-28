#![allow(dead_code)]

use std::sync::{atomic::AtomicBool, Arc, Mutex};

use crate::{
    entity::{Bullet, Cannon, Enemy, Point},
    TOTAL_AIS,
};

pub struct SharedResources {
    pub is_running: Arc<AtomicBool>,
    pub is_real_time: Arc<AtomicBool>,
    pub dimensions: Arc<Mutex<Point>>,
    pub selected_ai: Arc<Mutex<usize>>,
    pub cannons: Arc<Mutex<[Cannon; TOTAL_AIS]>>,
    pub bullets: Arc<Mutex<[Vec<Bullet>; TOTAL_AIS]>>,
    pub enemies: Arc<Mutex<[Vec<Enemy>; TOTAL_AIS]>>,
}

impl SharedResources {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(true)),
            is_real_time: Arc::new(AtomicBool::new(true)),
            dimensions: Arc::new(Mutex::new(Point { x: 800.0, y: 600.0 })),
            selected_ai: Arc::new(Mutex::new(0)),
            cannons: Arc::new(Mutex::new(std::array::from_fn(|_| Cannon::new()))),
            bullets: Arc::new(Mutex::new(std::array::from_fn(|_| vec![]))),
            enemies: Arc::new(Mutex::new(std::array::from_fn(|_| vec![]))),
        }
    }
}
