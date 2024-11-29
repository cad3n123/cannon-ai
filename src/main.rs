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
mod neural_network;
mod ui;

use entity::{
    Bullet, Cannon, Enemy, Entity, Point, Sprite, BARREL_HEIGHT, BULLET_HEIGHT, CANNON_RADIUS,
    ENEMY_HEIGHT,
};
use multi_threading::SharedResources;
use na::DVector;
use neural_network::NeuralNetwork;
use rand::Rng;
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle};
use std::{
    borrow::Borrow,
    cell::RefCell,
    f32::consts::PI,
    io,
    num::NonZero,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, available_parallelism, JoinHandle},
    time::Instant,
};
use typed_floats::Positive;
use ui::Button;

const TWO_PI: f32 = 2.0 * PI;
const VIEW_RAY_LENGTH: usize = 400;
const GUN_ROTATE_VELOCITY: f32 = 0.75;
const BULLET_SPEED: f32 = 100.0;
const BULLET_COOLDOWN: f32 = 2.0;
const ENEMY_COOLDOWN: f32 = 5.0;
const ENEMY_SPEED: f32 = 25.0;
const ENEMY_SPAWN_DISTANCE: usize = 1;

const SIMULATION_DELTA_TIME: f32 = 0.5;
const TRAINING_TIME: f32 = 40.0;
const MAX_TWEAK_CHANGE: f32 = 0.05;

fn main() {
    run_cannon_ai();
}
fn run_cannon_ai() -> Result<(), io::Error> {
    let shared_resources = SharedResources::new()?;

    let simulation = run_simulation(shared_resources.clone());

    run_display(shared_resources.clone());

    simulation.join().expect("Simulation panicked");

    println!("Program exiting gracefully");
    Ok(())
}
fn run_display(shared_resources: SharedResources) {
    let (mut rl, thread) = start_raylib();
    let mut buttons = create_buttons(
        &shared_resources.total_ais,
        &shared_resources.is_real_time,
        &shared_resources.selected_ai,
    );

    while !rl.window_should_close() {
        if rl.is_window_resized() {
            update_dimensions(&rl, &shared_resources.dimensions, &shared_resources.cannons);
        }

        let mut d = rl.begin_drawing(&thread);
        update_display(
            &mut d,
            &shared_resources.selected_ai,
            &mut buttons,
            &shared_resources.cannons,
            &shared_resources.enemies,
            &shared_resources.bullets,
        );
        for button in buttons.iter_mut() {
            button.borrow_mut().update(&d);
        }
    }

    drop(rl);
    shared_resources.is_running.store(false, Ordering::SeqCst);
}

fn create_buttons(
    total_ais_clone: &Arc<NonZero<usize>>,
    is_real_time: &Arc<AtomicBool>,
    selected_ai_clone: &Arc<Mutex<usize>>,
) -> Box<[Rc<RefCell<Button>>]> {
    let selected_ai = {
        let lock = lock_with_error!(selected_ai_clone);
        *lock
    };
    let mut decrement_selected_ai_button: Option<Rc<RefCell<Button>>> = None;
    let mut increment_selected_ai_button: Option<Rc<RefCell<Button>>> = None;
    decrement_selected_ai_button = Some(regular_button!(
        if selected_ai == 0 { " " } else { "<" },
        Point { x: 5.0, y: 5.0 },
        {
            let selected_ai_clone = Arc::clone(selected_ai_clone);
            let increment_selected_ai_button = increment_selected_ai_button.clone();
            Box::new(move |self_: &mut Button| {
                let mut selected_ai = lock_with_error!(selected_ai_clone);
                if *selected_ai > 0 {
                    *selected_ai -= 1;
                }
                if *selected_ai == 0 {
                    self_.text = " ".to_string();
                }
                if let Some(ref button) = increment_selected_ai_button {
                    button.borrow_mut().text = ">".to_string();
                }
            })
        }
    ));
    increment_selected_ai_button = Some(regular_button!(
        {
            if selected_ai == Into::<usize>::into(**total_ais_clone) - 1 {
                " "
            } else {
                ">"
            }
        },
        Point { x: 25.0, y: 5.0 },
        {
            let total_ais_clone = Arc::clone(total_ais_clone);
            let selected_ai_clone = Arc::clone(selected_ai_clone);
            let decrement_selected_ai_button = decrement_selected_ai_button.clone();
            Box::new(move |self_: &mut Button| {
                let mut selected_ai = lock_with_error!(selected_ai_clone);
                *selected_ai += 1;
                if *selected_ai >= Into::<usize>::into(*total_ais_clone) - 1 {
                    *selected_ai = Into::<usize>::into(*total_ais_clone) - 1;
                    self_.text = " ".to_string();
                }
                if let Some(ref button) = decrement_selected_ai_button {
                    button.borrow_mut().text = "<".to_string();
                }
            })
        }
    ));
    vec![
        decrement_selected_ai_button.unwrap(),
        increment_selected_ai_button.unwrap(),
        regular_button!("Speed Up", Point { x: 5.0, y: 30.0 }, {
            let is_real_time = Arc::clone(is_real_time);
            Box::new(move |self_: &mut Button| {
                let current_state = is_real_time.load(Ordering::SeqCst);
                is_real_time.store(!current_state, Ordering::SeqCst);

                if is_real_time.load(Ordering::SeqCst) {
                    self_.text = "Speed Up".to_string();
                } else {
                    self_.text = "Slow Down".to_string();
                }
            })
        }),
    ]
    .into_boxed_slice()
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
    d: &mut raylib::prelude::RaylibDrawHandle<'_>,
    selected_ai: &Arc<Mutex<usize>>,
    buttons: &mut Box<[Rc<RefCell<Button>>]>,
    cannons: &Arc<Mutex<Box<[Cannon]>>>,
    enemies: &Arc<Mutex<Box<[Vec<Enemy>]>>>,
    bullets: &Arc<Mutex<Box<[Vec<Bullet>]>>>,
) {
    d.clear_background(Color::RAYWHITE);

    draw_buttons(buttons, d);
    draw_entities(selected_ai, cannons, d, enemies, bullets);
}

fn draw_buttons(
    buttons: &mut Box<[Rc<RefCell<Button>>]>,
    d: &mut raylib::prelude::RaylibDrawHandle<'_>,
) {
    for button in buttons.iter_mut() {
        button.borrow_mut().draw(d);
    }
}

fn draw_entities(
    selected_ai: &Arc<Mutex<usize>>,
    cannons: &Arc<Mutex<Box<[Cannon]>>>,
    d: &mut raylib::prelude::RaylibDrawHandle<'_>,
    enemies: &Arc<Mutex<Box<[Vec<Enemy>]>>>,
    bullets: &Arc<Mutex<Box<[Vec<Bullet>]>>>,
) {
    let selected_ai = lock_with_error!(selected_ai);
    {
        let cannons = lock_with_error!(cannons);
        cannons[*selected_ai].draw(d);
    }
    {
        let bullets = &lock_with_error!(bullets)[*selected_ai];
        for bullet in bullets {
            bullet.draw(d);
        }
    }
    {
        let enemies = &lock_with_error!(enemies)[*selected_ai];
        for enemy in enemies {
            enemy.draw(d);
        }
    }
}
fn update_dimensions(
    rl: &RaylibHandle,
    dimensions: &Arc<Mutex<Point>>,
    cannons: &Arc<Mutex<Box<[Cannon]>>>,
) {
    let mut dims = lock_with_error!(dimensions);
    let width = rl.get_render_width() as f32;
    let height = rl.get_render_height() as f32;
    dims.x = width;
    dims.y = height;
    drop(dims);

    let mut cannons = lock_with_error!(cannons);
    for cannon in cannons.iter_mut() {
        cannon.position.x = width / 2.0;
        cannon.position.y = height / 2.0;
    }
}

fn run_simulation(shared_resources: SharedResources) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut ai_threads: Vec<JoinHandle<()>> = vec![];
        for ai_index in 0..Into::<usize>::into(*shared_resources.total_ais) {
            let shared_resources_clone = shared_resources.arc_clone();
            let mut time_since_enemy: f32 = ENEMY_COOLDOWN - 2.0;
            let mut time_since_bullet = 0.0_f32;

            ai_threads.push(thread::spawn(move || {
                let mut last_time = Instant::now();

                while shared_resources_clone.is_running.load(Ordering::SeqCst) {
                    let now = Instant::now();
                    let delta_time = if shared_resources_clone.is_real_time.load(Ordering::SeqCst) {
                        now.duration_since(last_time).as_secs_f32()
                    } else {
                        0.001
                    };
                    last_time = now;

                    time_since_enemy += delta_time;
                    time_since_bullet += delta_time;

                    destroy_entities(
                        &shared_resources_clone.dimensions,
                        &shared_resources_clone.enemies,
                        ai_index,
                        &shared_resources_clone.bullets,
                    );
                    create_entities(
                        &mut time_since_enemy,
                        ai_index,
                        &mut time_since_bullet,
                        &shared_resources_clone.dimensions,
                        &shared_resources_clone.cannons,
                        &shared_resources_clone.bullets,
                        &shared_resources_clone.enemies,
                    );
                    update_entites(
                        &shared_resources_clone.cannons,
                        ai_index,
                        delta_time,
                        &shared_resources_clone.enemies,
                        &shared_resources_clone.bullets,
                    );
                }
            }));
        }

        for handle in ai_threads {
            handle.join().expect("AI thread panicked");
        }
    })
}

fn destroy_entities(
    shared_dimensions: &Arc<Mutex<Point>>,
    shared_enemies: &Arc<Mutex<Box<[Vec<Enemy>]>>>,
    ai_index: usize,
    shared_bullets: &Arc<Mutex<Box<[Vec<Bullet>]>>>,
) {
    let locked_dimensions = lock_with_error!(shared_dimensions);
    let dimensions = locked_dimensions.clone();
    drop(locked_dimensions);
    let enemies = &mut lock_with_error!(shared_enemies)[ai_index];
    {
        let bullets = &mut lock_with_error!(shared_bullets)[ai_index];
        let mut i = 0;
        'bullet: while i < bullets.len() {
            let bullet_pos = &bullets[i].position;
            if bullet_pos.x < 0.0
                || bullet_pos.x > dimensions.x
                || bullet_pos.y < 0.0
                || bullet_pos.y > dimensions.y
            {
                bullets.remove(i);
            } else {
                let mut j = 0;
                while j < enemies.len() {
                    let enemy_pos = &enemies[j].position;
                    if bullet_pos.difference(enemy_pos).magnitude()
                        <= (BULLET_HEIGHT + ENEMY_HEIGHT) / 2.0
                    {
                        bullets.remove(i);
                        enemies.remove(j);
                        continue 'bullet;
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
    }
    let mut i = 0;
    while i < enemies.len() {
        let center_offset = enemies[i].position.difference(&Point {
            x: dimensions.x / 2.0,
            y: dimensions.y / 2.0,
        });
        let center_distance = center_offset.magnitude();
        if center_distance < CANNON_RADIUS + ENEMY_HEIGHT / 2.0 {
            enemies.remove(i);
        } else {
            i += 1;
        }
    }
}

fn create_entities(
    time_since_enemy: &mut f32,
    ai_index: usize,
    time_since_bullet: &mut f32,
    dimensions: &Arc<Mutex<Point>>,
    cannons: &Arc<Mutex<Box<[Cannon]>>>,
    bullets: &Arc<Mutex<Box<[Vec<Bullet>]>>>,
    enemies: &Arc<Mutex<Box<[Vec<Enemy>]>>>,
) {
    if *time_since_enemy >= ENEMY_COOLDOWN {
        *time_since_enemy = 0.0;
        spawn_rand_enemy(enemies, ai_index, dimensions);
    }
    if *time_since_bullet >= BULLET_COOLDOWN {
        *time_since_bullet = 0.0;
        spawn_bullet(cannons, ai_index, bullets, dimensions);
    }
}

fn spawn_bullet(
    cannons_clone: &Arc<Mutex<Box<[Cannon]>>>,
    ai_index: usize,
    bullets_clone: &Arc<Mutex<Box<[Vec<Bullet>]>>>,
    dimensions_clone: &Arc<Mutex<Point>>,
) {
    let direction = lock_with_error!(cannons_clone)[ai_index].direction;
    let (direction_cos, direction_sin) = (direction.cos(), direction.sin());
    let (center_x, center_y) = get_center(dimensions_clone);
    let bullets = &mut lock_with_error!(bullets_clone)[ai_index];
    bullets.push(Bullet {
        position: Point {
            x: center_x + direction_cos * (CANNON_RADIUS + BARREL_HEIGHT),
            y: center_y + direction_sin * (CANNON_RADIUS + BARREL_HEIGHT),
        },
        direction,
        velocity: Point {
            x: direction_cos * BULLET_SPEED,
            y: direction_sin * BULLET_SPEED,
        },
    });
}

fn spawn_rand_enemy(
    enemies_clone: &Arc<Mutex<Box<[Vec<Enemy>]>>>,
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
    cannons_clone: &Arc<Mutex<Box<[Cannon]>>>,
    ai_index: usize,
    delta_time: f32,
    enemies_clone: &Arc<Mutex<Box<[Vec<Enemy>]>>>,
    bullets_clone: &Arc<Mutex<Box<[Vec<Bullet>]>>>,
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
fn find_largest_index_unchecked(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(index, _)| index)
        .unwrap()
}
fn find_n_lowest_indices(values: &[f32], n: usize) -> Box<[usize]> {
    // Create a vector of indices paired with their corresponding values.
    let mut indexed_values: Vec<(usize, f32)> = values.iter().cloned().enumerate().collect();

    // Sort the vector by the values (second element of the tuple).
    indexed_values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Extract the indices of the n lowest values.
    indexed_values
        .iter()
        .take(n)
        .map(|&(index, _)| index)
        .collect::<Vec<usize>>()
        .into_boxed_slice()
}
