macro_rules! lock_with_error {
    ($var:ident) => {
        $var.lock()
            .expect(&format!("Failed to lock {} mutex", stringify!($var)))
    };
}
macro_rules! regular_button {
    ($text:expr, $position:expr, $on_click_up:expr) => {
        Rc::new(RefCell::new(Button::build(
            $text.to_string(),
            &$position,
            Color::BLACK,
            None,
            Some(Box::new(|button: &mut Button| {
                button.font_color = Color {
                    r: 80,
                    g: 80,
                    b: 80,
                    a: 255,
                };
            })),
            Some(Box::new(|button: &mut Button| {
                button.font_color = Color {
                    r: 50,
                    g: 50,
                    b: 50,
                    a: 255,
                };
            })),
            Some(Box::new(|button: &mut Button| {
                button.font_color = Color::BLACK;
            })),
            Some($on_click_up),
        )))
    };
}

mod entity;
mod multi_threading;
mod ui;

use entity::{Bullet, Cannon, Enemy, Entity, Point, Sprite};
use multi_threading::SharedResources;
use rand::Rng;
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle};
use std::{
    cell::RefCell,
    f32::consts::PI,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};
use ui::Button;

const TWO_PI: f32 = 2.0 * PI;
const VIEW_RAY_LENGTH: usize = 400;
const GUN_ROTATE_VELOCITY: f32 = 0.75;
const BULLET_SPEED: f32 = 100.0;
const BULLET_COOLDOWN: f32 = 2.0;
const ENEMY_COOLDOWN: f32 = 5.0;
const ENEMY_SPEED: f32 = 25.0;
const ENEMY_SPAWN_DISTANCE: usize = 1;
pub const TOTAL_AIS: usize = 10;
const SIMULATION_DELTA_TIME: f32 = 0.5;
const TRAINING_TIME: f32 = 40.0;
const MAX_TWEAK_CHANGE: f32 = 0.05;

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
        let speed_up_button = regular_button!("Speed Up", Point { x: 0.0, y: 0.0 }, {
            let is_real_time = Arc::clone(&is_real_time);
            Box::new(move |speed_up_button: &mut Button| {
                let current_state = is_real_time.load(Ordering::SeqCst);
                is_real_time.store(!current_state, Ordering::SeqCst);

                if is_real_time.load(Ordering::SeqCst) {
                    speed_up_button.text = "Speed Up".to_string();
                } else {
                    speed_up_button.text = "Slow Down".to_string();
                }
            })
        });
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
        let bullets = &lock_with_error!(bullets)[*selected_ai];
        for bullet in bullets {
            bullet.draw(&mut d);
        }
    }
    {
        let enemies = &lock_with_error!(enemies)[*selected_ai];
        for enemy in enemies {
            enemy.draw(&mut d);
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
    let width = rl.get_render_width() as f32;
    let height = rl.get_render_height() as f32;
    dims.x = width;
    dims.y = height;
    drop(dims);

    let selected_ai = lock_with_error!(selected_ai);
    let mut cannons = lock_with_error!(cannons);
    cannons[*selected_ai].position.x = width / 2.0;
    cannons[*selected_ai].position.y = height / 2.0;
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
            let dimensions_clone = Arc::clone(&dimensions);
            let cannons_clone = Arc::clone(&cannons);
            let enemies_clone = Arc::clone(&enemies);
            let bullets_clone = Arc::clone(&bullets);
            let mut time_since_enemy: f32 = ENEMY_COOLDOWN - 2.0;
            let mut time_since_bullet = 0.0_f32;

            ai_threads.push(thread::spawn(move || {
                let mut last_time = Instant::now();

                while is_running_clone.load(Ordering::SeqCst) {
                    let now = Instant::now();
                    let delta_time = now.duration_since(last_time).as_secs_f32();
                    last_time = now;

                    time_since_enemy += delta_time;
                    time_since_bullet += delta_time;

                    if time_since_enemy >= ENEMY_COOLDOWN {
                        time_since_enemy = 0.0;
                        spawn_rand_enemy(&enemies_clone, ai_index, &dimensions_clone);
                    }
                    if time_since_bullet >= BULLET_COOLDOWN {
                        time_since_bullet = 0.0;
                        spawn_bullet(&cannons_clone, ai_index, &bullets_clone, &dimensions_clone);
                    }

                    update_entites(
                        &cannons_clone,
                        ai_index,
                        delta_time,
                        &enemies_clone,
                        &bullets_clone,
                    );
                }
            }));
        }

        for handle in ai_threads {
            handle.join().expect("AI thread panicked");
        }
    })
}

fn spawn_bullet(
    cannons_clone: &Arc<Mutex<[Cannon; 10]>>,
    ai_index: usize,
    bullets_clone: &Arc<Mutex<[Vec<Bullet>; 10]>>,
    dimensions_clone: &Arc<Mutex<Point>>,
) {
    let cannons = lock_with_error!(cannons_clone);
    let direction = cannons[ai_index].direction;
    drop(cannons);
    let (center_x, center_y) = get_center(dimensions_clone);
    let bullets = &mut lock_with_error!(bullets_clone)[ai_index];
    bullets.push(Bullet {
        position: Point {
            x: center_x,
            y: center_y,
        },
        direction,
        velocity: Point {
            x: direction.cos() * BULLET_SPEED,
            y: direction.sin() * BULLET_SPEED,
        },
    });
}

fn spawn_rand_enemy(
    enemies_clone: &Arc<Mutex<[Vec<Enemy>; 10]>>,
    ai_index: usize,
    dimensions_clone: &Arc<Mutex<Point>>,
) {
    let (center_x, center_y) = get_center(dimensions_clone);
    let location_direction = rand::thread_rng().gen_range(0.0..TWO_PI);
    let facing_direction = PI + location_direction;
    let enemies = &mut lock_with_error!(enemies_clone)[ai_index];
    enemies.push(Enemy {
        position: Point {
            x: center_x
                + location_direction.cos() * (VIEW_RAY_LENGTH + ENEMY_SPAWN_DISTANCE) as f32,
            y: center_y
                + location_direction.sin() * (VIEW_RAY_LENGTH + ENEMY_SPAWN_DISTANCE) as f32,
        },
        direction: facing_direction,
        velocity: Point {
            x: ENEMY_SPEED * facing_direction.cos(),
            y: ENEMY_SPEED * facing_direction.sin(),
        },
    });
}

fn get_center(dimensions_clone: &Arc<Mutex<Point>>) -> (f32, f32) {
    let dimensions = lock_with_error!(dimensions_clone);
    let (center_x, center_y) = (dimensions.x / 2.0, dimensions.y / 2.0);
    (center_x, center_y)
}

fn update_entites(
    cannons_clone: &Arc<Mutex<[Cannon; 10]>>,
    ai_index: usize,
    delta_time: f32,
    enemies_clone: &Arc<Mutex<[Vec<Enemy>; 10]>>,
    bullets_clone: &Arc<Mutex<[Vec<Bullet>; 10]>>,
) {
    {
        let mut cannons = lock_with_error!(cannons_clone);
        cannons[ai_index].direction += (ai_index + 1) as f32 * delta_time;
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
