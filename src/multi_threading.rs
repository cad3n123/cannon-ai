#![allow(dead_code)]

macro_rules! new_arc_mutex {
    ($val:expr) => {
        std::sync::Arc::new(std::sync::Mutex::new($val))
    };
}
macro_rules! new_arc_atomic_bool {
    ($val:expr) => {
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new($val))
    };
}
macro_rules! new_dynamic_array {
    ($x:expr, $y:expr, $z:ty) => {
        (0..$x).map(|_| $y).collect::<Vec<$z>>().into_boxed_slice()
    };
}

use std::{io, num::NonZero, sync::{atomic::AtomicBool, Arc, Mutex}, thread::available_parallelism};

use crate::entity::{Bullet, Cannon, Enemy, Point};

pub struct SharedResources {
    pub total_ais: Arc<NonZero<usize>>,
    pub is_running: Arc<AtomicBool>,
    pub is_real_time: Arc<AtomicBool>,
    pub dimensions: Arc<Mutex<Point>>,
    pub selected_ai: Arc<Mutex<usize>>,
    pub cannons: Arc<Mutex<Box<[Cannon]>>>,
    pub bullets: Arc<Mutex<Box<[Vec<Bullet>]>>>,
    pub enemies: Arc<Mutex<Box<[Vec<Enemy>]>>>,
}

impl SharedResources {
    pub fn new() -> Result<Self, io::Error> {
        let total_ais = {
            let total_ais = available_parallelism()?;
            if Into::<usize>::into(total_ais) % 2 == 0 {
                total_ais
            } else if Into::<usize>::into(total_ais) < 10 {
                NonZero::new(10_usize).expect("Computational error")
            } else {
                NonZero::new(Into::<usize>::into(total_ais) - 1).expect("Computational error")
            }
        };
        Ok(Self {
            total_ais: Arc::new(total_ais),
            is_running: new_arc_atomic_bool!(true),
            is_real_time: new_arc_atomic_bool!(true),
            dimensions: new_arc_mutex!(Point { x: 800.0, y: 600.0 }),
            selected_ai: new_arc_mutex!(0),
            cannons: new_arc_mutex!(new_dynamic_array!(total_ais.into(), Cannon::new(), Cannon)),
            bullets: new_arc_mutex!(new_dynamic_array!(total_ais.into(), vec![], Vec<Bullet>)),
            enemies: new_arc_mutex!(new_dynamic_array!(total_ais.into(), vec![], Vec<Enemy>)),
        })
    }
}
