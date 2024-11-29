#![allow(dead_code)]

#[macro_export]
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
#[macro_export]
macro_rules! new_dynamic_array {
    ($x:expr, $y:expr, $z:ty) => {
        (0..$x).map(|_| $y).collect::<Vec<$z>>().into_boxed_slice()
    };
}

use std::{
    fs::File,
    io::{self, Read, Write},
    num::NonZero,
    path::Path,
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::available_parallelism,
};

use crate::{
    entity::{Bullet, Cannon, Enemy, Point},
    neural_network::NeuralNetwork,
    TOTAL_VIEW_RAYS,
};

#[derive(Clone)]
pub struct SharedResources {
    pub total_ais: Arc<NonZero<usize>>,
    pub is_running: Arc<AtomicBool>,
    pub is_real_time: Arc<AtomicBool>,
    pub dimensions: Arc<Mutex<Point>>,
    pub elapsed_simulation_times: Arc<Mutex<Box<[f32]>>>,
    pub selected_ai: Arc<Mutex<usize>>,
    pub ai_scores: Arc<Mutex<Box<[f32]>>>,
    pub direction_ais: Arc<Mutex<Box<[NeuralNetwork]>>>,
    pub shooting_ais: Arc<Mutex<Box<[NeuralNetwork]>>>,
    pub cannons: Arc<Mutex<Box<[Cannon]>>>,
    pub bullets: Arc<Mutex<Box<[Vec<Bullet>]>>>,
    pub enemies: Arc<Mutex<Box<[Vec<Enemy>]>>>,
}

impl SharedResources {
    pub fn new() -> Result<Self, io::Error> {
        let total_ais = {
            let mut total_ais = available_parallelism()?;
            if total_ais > unsafe { NonZero::new_unchecked(1) } {
                total_ais = unsafe { NonZero::new_unchecked(Into::<usize>::into(total_ais) - 1) };
            }
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
            elapsed_simulation_times: new_arc_mutex!(new_dynamic_array!(
                total_ais.into(),
                0.0,
                f32
            )),
            selected_ai: new_arc_mutex!(0),
            ai_scores: new_arc_mutex!(new_dynamic_array!(total_ais.into(), 0.0, f32)),
            direction_ais: {
                let file_name = format!("direction_ais_{}.json", Into::<usize>::into(total_ais));
                if Path::new(file_name.as_str()).exists() {
                    let mut file = File::open(file_name.clone())?;
                    let mut json = String::new();
                    file.read_to_string(&mut json)?;
                    new_arc_mutex!(serde_json::from_str(&json)
                        .unwrap_or_else(|_| panic!("Cannot read file {file_name}")))
                } else {
                    new_arc_mutex!(new_dynamic_array!(
                        total_ais.into(),
                        NeuralNetwork::new_random_unchecked(&[
                            TOTAL_VIEW_RAYS,
                            TOTAL_VIEW_RAYS / 2,
                            3
                        ]),
                        NeuralNetwork
                    ))
                }
            },
            shooting_ais: {
                let file_name = format!("shooting_ais_{}.json", Into::<usize>::into(total_ais));
                if Path::new(file_name.as_str()).exists() {
                    let mut file = File::open(file_name.clone())?;
                    let mut json = String::new();
                    file.read_to_string(&mut json)?;
                    new_arc_mutex!(serde_json::from_str(&json)
                        .unwrap_or_else(|_| panic!("Cannot read file {file_name}")))
                } else {
                    new_arc_mutex!(new_dynamic_array!(
                        total_ais.into(),
                        NeuralNetwork::new_random_unchecked(&[
                            TOTAL_VIEW_RAYS,
                            TOTAL_VIEW_RAYS / 2,
                            2
                        ]),
                        NeuralNetwork
                    ))
                }
            },
            cannons: new_arc_mutex!(new_dynamic_array!(total_ais.into(), Cannon::new(), Cannon)),
            bullets: new_arc_mutex!(new_dynamic_array!(total_ais.into(), vec![], Vec<Bullet>)),
            enemies: new_arc_mutex!(new_dynamic_array!(total_ais.into(), vec![], Vec<Enemy>)),
        })
    }
    pub fn arc_clone(&self) -> Self {
        Self {
            total_ais: Arc::clone(&self.total_ais),
            is_running: Arc::clone(&self.is_running),
            is_real_time: Arc::clone(&self.is_real_time),
            dimensions: Arc::clone(&self.dimensions),
            elapsed_simulation_times: Arc::clone(&self.elapsed_simulation_times),
            selected_ai: Arc::clone(&self.selected_ai),
            ai_scores: Arc::clone(&self.ai_scores),
            direction_ais: Arc::clone(&self.direction_ais),
            shooting_ais: Arc::clone(&self.shooting_ais),
            cannons: Arc::clone(&self.cannons),
            bullets: Arc::clone(&self.bullets),
            enemies: Arc::clone(&self.enemies),
        }
    }
    pub fn save_ais(&self) -> Result<(), io::Error> {
        let total_ais = Into::<usize>::into(*self.total_ais);

        let direction_json =
            serde_json::to_string_pretty(&lock_with_error!(self.direction_ais).clone()).unwrap();
        let file_name = format!("direction_ais_{}.json", total_ais);
        let mut file = File::create(file_name)?;
        file.write_all(direction_json.as_bytes())?;

        let shooting_json =
            serde_json::to_string_pretty(&lock_with_error!(self.shooting_ais).clone()).unwrap();
        let file_name = format!("shooting_ais_{}.json", total_ais);
        let mut file = File::create(file_name)?;
        file.write_all(shooting_json.as_bytes())?;
        Ok(())
    }
}
