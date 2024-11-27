macro_rules! lock_with_error {
    ($var:ident) => {
        $var.lock()
            .expect(&format!("Failed to lock {} mutex", stringify!($var)))
    };
}

mod entity;
mod multi_threading;
mod ui;

use entity::{Bullet, Cannon, Enemy, Entity, Point, Sprite};
use multi_threading::SharedResources;
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};
use ui::Button;

pub const TOTAL_AIS: usize = 10;

fn main() {
    let shared_resources = SharedResources::new();

    let simulation = run_simulation(
        shared_resources.is_running.clone(),
        shared_resources.dimensions.clone(),
        shared_resources.cannons.clone(),
        shared_resources.enemies.clone(),
        shared_resources.bullets.clone(),
    );

    run_display(
        shared_resources.is_running.clone(),
        shared_resources.is_real_time.clone(),
        shared_resources.dimensions.clone(),
        shared_resources.selected_ai.clone(),
        shared_resources.cannons.clone(),
        shared_resources.enemies.clone(),
        shared_resources.bullets.clone(),
    );

    simulation.join().expect("Simulation panicked");

    println!("Program exiting gracefully");
}
fn run_display(
    is_running: Arc<AtomicBool>,
    is_real_time: Arc<AtomicBool>,
    dimensions: Arc<Mutex<Point>>,
    selected_ai: Arc<Mutex<usize>>,
    cannons: Arc<Mutex<[Cannon; TOTAL_AIS]>>,
    enemies: Arc<Mutex<[Vec<Enemy>; TOTAL_AIS]>>,
    bullets: Arc<Mutex<[Vec<Bullet>; TOTAL_AIS]>>,
) {
    let (mut rl, thread) = start_raylib();
    let buttons = {
        let speed_up_button: Rc<RefCell<Button>> = Rc::new(RefCell::new(Button::build(
            "Speed Up".to_string(),
            &Point { x: 0.0, y: 0.0 },
            Color::BLACK,
            None,
            Some(Box::new(|speed_up_button: &mut Button| {
                speed_up_button.font_color = Color {
                    r: 80,
                    g: 80,
                    b: 80,
                    a: 255,
                };
            })),
            Some(Box::new(|speed_up_button: &mut Button| {
                speed_up_button.font_color = Color {
                    r: 50,
                    g: 50,
                    b: 50,
                    a: 255,
                };
            })),
            Some(Box::new(|speed_up_button: &mut Button| {
                speed_up_button.font_color = Color::BLACK;
            })),
            Some({
                let is_real_time = Arc::clone(&is_real_time); // Clone Arc for use in the closure
                Box::new(move |speed_up_button: &mut Button| {
                    let current_state = is_real_time.load(Ordering::SeqCst);
                    is_real_time.store(!current_state, Ordering::SeqCst); // Toggle the boolean value

                    if is_real_time.load(Ordering::SeqCst) {
                        speed_up_button.text = "Speed Up".to_string();
                    } else {
                        speed_up_button.text = "Slow Down".to_string();
                    }
                })
            }),
        )));
        [speed_up_button]
    };

    while !rl.window_should_close() {
        if rl.is_window_resized() {
            update_dimensions(&rl, &dimensions, &selected_ai, &cannons);
        }

        let d = rl.begin_drawing(&thread);
        update_display(d, &selected_ai, &cannons, &enemies, &bullets);
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
    enemies: &Arc<Mutex<[Vec<Enemy>; TOTAL_AIS]>>,
    bullets: &Arc<Mutex<[Vec<Bullet>; TOTAL_AIS]>>,
) {
    d.clear_background(Color::RAYWHITE);

    draw_entities(selected_ai, cannons, d, enemies, bullets);
}

fn draw_entities(
    selected_ai: &Arc<Mutex<usize>>,
    cannons: &Arc<Mutex<[Cannon; 10]>>,
    mut d: raylib::prelude::RaylibDrawHandle<'_>,
    enemies: &Arc<Mutex<[Vec<Enemy>; 10]>>,
    bullets: &Arc<Mutex<[Vec<Bullet>; 10]>>,
) {
    let selected_ai = lock_with_error!(selected_ai);
    {
        let cannons = lock_with_error!(cannons);
        cannons[*selected_ai].draw(&mut d);
    }
    {
        let enemies = &lock_with_error!(enemies)[*selected_ai];
        for enemy in enemies {
            enemy.draw(&mut d);
        }
    }
    {
        let bullets = &lock_with_error!(bullets)[*selected_ai];
        for bullet in bullets {
            bullet.draw(&mut d);
        }
    }
}
fn update_dimensions(
    rl: &RaylibHandle,
    dimensions: &Arc<Mutex<Point>>,
    selected_ai: &Arc<Mutex<usize>>,
    cannons: &Arc<Mutex<[Cannon; TOTAL_AIS]>>,
) {
    let mut dims = lock_with_error!(dimensions);
    dims.x = rl.get_render_width() as f32;
    dims.y = rl.get_render_height() as f32;

    let selected_ai = lock_with_error!(selected_ai);
    let mut cannons = lock_with_error!(cannons);
    cannons[*selected_ai].position.x = dims.x / 2.0;
    cannons[*selected_ai].position.y = dims.y / 2.0;
}

fn run_simulation(
    is_running: Arc<AtomicBool>,
    dimensions: Arc<Mutex<Point>>,
    cannons: Arc<Mutex<[Cannon; TOTAL_AIS]>>,
    enemies: Arc<Mutex<[Vec<Enemy>; TOTAL_AIS]>>,
    bullets: Arc<Mutex<[Vec<Bullet>; TOTAL_AIS]>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut ai_threads: Vec<JoinHandle<()>> = vec![];
        for ai_index in 0..TOTAL_AIS {
            let is_running_clone = Arc::clone(&is_running);
            let cannons_clone = Arc::clone(&cannons);
            let enemies_clone = Arc::clone(&enemies);
            let bullets_clone = Arc::clone(&bullets);

            ai_threads.push(thread::spawn(move || {
                let mut last_time = Instant::now();

                while is_running_clone.load(Ordering::SeqCst) {
                    let now = Instant::now();
                    let delta_time = now.duration_since(last_time).as_secs_f32();
                    last_time = now;

                    {
                        let mut cannons = lock_with_error!(cannons_clone);
                        cannons[ai_index].direction += 1.0 * delta_time;
                    }
                    {
                        let enemies = &mut lock_with_error!(enemies_clone)[ai_index];
                        for enemy in enemies {
                            enemy.update(delta_time);
                        }
                    }
                    {
                        let bullets = &mut lock_with_error!(bullets_clone)[ai_index];
                        for bullet in bullets {
                            bullet.update(delta_time);
                        }
                    }
                }
            }));
        }

        for handle in ai_threads {
            handle.join().expect("AI thread panicked");
        }
    })
}
