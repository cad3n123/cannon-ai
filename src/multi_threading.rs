use std::sync::{atomic::AtomicBool, Arc, Mutex};

use crate::{entity::{Cannon, Point}, TOTAL_AIS};

pub struct SharedResources {
    pub is_running: Arc<AtomicBool>,
    pub dimensions: Arc<Mutex<Point>>,
    pub cannons: Arc<Mutex<[Cannon; TOTAL_AIS]>>,
    pub selected_ai: Arc<Mutex<usize>>,
}

impl SharedResources {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(true)),
            dimensions: Arc::new(Mutex::new(Point { x: 800.0, y: 600.0 })),
            cannons: Arc::new(Mutex::new(std::array::from_fn(|_| Cannon::new()))),
            selected_ai: Arc::new(Mutex::new(0)),
        }
    }
}