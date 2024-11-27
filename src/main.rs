mod entity;
mod multi_threading;

use entity::{Bullet, Cannon, Enemy, Point, Sprite};
use multi_threading::SharedResources;
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub const TOTAL_AIS: usize = 10;

fn main() {
    let shared_resources = SharedResources::new();

    let simulation = run_simulation(
        shared_resources.is_running.clone(),
        shared_resources.dimensions.clone(),
        shared_resources.selected_ai.clone(),
        shared_resources.cannons.clone(),
    );

    run_display(
        shared_resources.is_running.clone(),
        shared_resources.dimensions.clone(),
        shared_resources.selected_ai.clone(),
        shared_resources.cannons.clone(),
    );

    simulation.join().expect("Simulation panicked");

    println!("Program exiting gracefully");
}
fn run_display(
    is_running: Arc<AtomicBool>,
    dimensions: Arc<Mutex<Point>>,
    selected_ai: Arc<Mutex<usize>>,
    cannons: Arc<Mutex<[Cannon; TOTAL_AIS]>>,
) {
    let (mut rl, thread) = start_raylib();

    while !rl.window_should_close() {
        if rl.is_window_resized() {
            update_dimensions(&rl, &dimensions, &selected_ai, &cannons);
        }

        let d = rl.begin_drawing(&thread);
        update_display(d, &selected_ai, &cannons);
    }

    drop(rl);
    is_running.store(false, Ordering::SeqCst);
}
fn start_raylib() -> (RaylibHandle, raylib::RaylibThread) {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("AI Cannon")
        .fullscreen()
        .build();

    rl.set_target_fps(60);
    (rl, thread)
}
fn update_display(
    mut d: raylib::prelude::RaylibDrawHandle<'_>,
    selected_ai: &Arc<Mutex<usize>>,
    cannons: &Arc<Mutex<[Cannon; TOTAL_AIS]>>,
) {
    d.clear_background(Color::RAYWHITE);

    {
        let selected_ai = selected_ai
            .lock()
            .expect("Failed to lock selected_ai mutex");
        let cannons = cannons.lock().expect("Failed to lock cannon mutex");
        cannons[*selected_ai].draw(&mut d);
    }
}
fn update_dimensions(
    rl: &RaylibHandle,
    dimensions: &Arc<Mutex<Point>>,
    selected_ai: &Arc<Mutex<usize>>,
    cannons: &Arc<Mutex<[Cannon; TOTAL_AIS]>>,
) {
    let mut dims = dimensions.lock().expect("Failed to lock dimensions mutex");
    dims.x = rl.get_render_width() as f32;
    dims.y = rl.get_render_height() as f32;

    let selected_ai = selected_ai.lock().expect("Failed to lock expected_ai");
    let mut cannons = cannons.lock().expect("Failed to lock cannon mutex");
    cannons[*selected_ai].position.x = dims.x / 2.0;
    cannons[*selected_ai].position.y = dims.y / 2.0;
}

fn run_simulation(
    is_running: Arc<AtomicBool>,
    dimensions: Arc<Mutex<Point>>,
    selected_ai: Arc<Mutex<usize>>,
    cannons: Arc<Mutex<[Cannon; TOTAL_AIS]>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut ai_threads: Vec<JoinHandle<()>> = vec![];
        for ai_index in 0..TOTAL_AIS {
            let is_running_clone = Arc::clone(&is_running);

            ai_threads.push(thread::spawn(move || {
                let mut last_time = Instant::now();

                while is_running_clone.load(Ordering::SeqCst) {
                    let now = Instant::now();
                    let delta_time = now.duration_since(last_time).as_secs_f32();
                    last_time = now;

                    {
                        let mut cannon = cannons.lock().expect("Failed to lock cannon mutex");
                        cannon[ai_index].direction += 1.0 * delta_time;
                    }
                }
            }));
        }

        for handle in ai_threads {
            handle.join().expect("AI thread panicked");
        }
    })
}