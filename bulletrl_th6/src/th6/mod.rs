use std::ffi::c_void;

use bulletrl_common::Vector2;
use log::{info, warn};
use rand::{rngs::ThreadRng, Rng};

use self::{
    offsets::{ENEMY_BULLETS, ENEMY_MANAGER, ENGINE, GAME, GAME_UI, ITEM_MANAGER, PLAYER},
    types::TouhouInput,
};

pub mod offsets;
pub mod patch;
pub mod types;

struct GlobalState {
    frame: u32,
    first_tick: bool,
    renderer: bulletrl_common::Renderer,
    client: Option<bulletrl_common::EnvClient>,
    training: bool,
    cur_input: bulletrl_common::Input,
    last_score: u32,
    rng: ThreadRng,

    #[cfg(feature = "renderer_debug")]
    window: minifb::Window,
}

static mut GLOBAL_STATE: *mut GlobalState = std::ptr::null_mut::<GlobalState>();

pub unsafe fn modify_game_settings() {
    std::ptr::write_bytes(GAME, 0, 1);

    (*GAME).difficulty = 1; // normal
    (*GAME).stage = (*GLOBAL_STATE).rng.gen_range(0..=5); // random stage
}

pub unsafe fn get_input() -> Option<u16> {
    // Don't overwrite input if there's no client to get input from
    if (*GLOBAL_STATE).client.is_none() {
        return None;
    }

    let mut th_input = TouhouInput::SHOOT | TouhouInput::SKIP_DIALOGUE;
    let input = (*GLOBAL_STATE).cur_input;

    if input.contains(bulletrl_common::Input::UP) {
        th_input |= TouhouInput::UP;
    }
    if input.contains(bulletrl_common::Input::DOWN) {
        th_input |= TouhouInput::DOWN;
    }
    if input.contains(bulletrl_common::Input::LEFT) {
        th_input |= TouhouInput::LEFT;
    }
    if input.contains(bulletrl_common::Input::RIGHT) {
        th_input |= TouhouInput::RIGHT;
    }
    if input.contains(bulletrl_common::Input::FOCUS) {
        th_input |= TouhouInput::FOCUS;
    }

    Some(th_input.bits())
}

unsafe extern "cdecl" fn custom_calc(state: *mut c_void) -> i32 {
    let mut state = (state as *mut GlobalState).as_mut().expect("getting state");

    // Only do custom calc stuff if in-game
    let done =
        (*GAME).game_over || (!(*GAME_UI).inner.is_null() && (*(*GAME_UI).inner).show_results == 1);
    if (*ENGINE).state == 2 {
        state.frame += 1;
        // Run at 15fps
        if state.frame % 4 != 0 && !done {
            return 1;
        }
    } else {
        return 1;
    }

    // Keep track of score differences between updates
    state.last_score = state.last_score.min((*GAME).score);
    let score_diff = (*GAME).score - state.last_score;
    state.last_score = (*GAME).score;
    if score_diff != 0 {
        info!("Score difference: {}", score_diff);
    }

    // Discard first input because input is being polled after sending observation, not the other way around
    // Training loop should be:   recv input -> tick game -> send observation -> repeat
    // Without this, it would be: recv input -> send observation -> tick game -> repeat
    if state.first_tick && let Some(client) = &mut state.client {
        client.recv_input().expect("receiving first input");
        state.first_tick = false;
    }

    // Build and send the observation
    render_observation(&mut state.renderer);

    // Render the observation to a window for debugging purposes
    #[cfg(feature = "renderer_debug")]
    state
        .window
        .update_with_buffer(
            &state.renderer.buffer,
            bulletrl_common::FIELD_WIDTH,
            bulletrl_common::FIELD_HEIGHT,
        )
        .expect("updating debug window");

    if let Some(client) = &mut state.client {
        // TODO: Refine reward shaping, this is lazy
        // 50% ln(score difference) / ln(10000)
        // 50% being within 50 units on the x-axis of an enemy
        let _score_reward = ((score_diff as f32).ln() / 10000.0f32.ln()).clamp(0.0, 1.0);
        let _distance_reward = {
            let mut max_val = None;
            for enemy in &(*ENEMY_MANAGER).enemies {
                if (enemy.enemy_type & 0x80) != 0 && (enemy.flags & 8) == 0 {
                    let enemy_reward = if (*PLAYER).pos.y > enemy.pos.y {
                        1.0 - (((*PLAYER).pos.x - enemy.pos.x).abs().clamp(0.0, 50.0) / 50.0)
                    } else {
                        0.0
                    };
                    if max_val.map(|x| enemy_reward > x).unwrap_or(true) {
                        max_val = Some(enemy_reward);
                    }
                }
            }

            // This is intentional, there should be full reward if there are no enemies left
            //max_val.unwrap_or(1.0)
            max_val.unwrap_or(0.0)
        };

        // Discourage hiding in the top of the screen near the start of training
        //let reward = (distance_reward + score_reward) / 2.0;
        /*
        let reward = if (*PLAYER).pos.y > bulletrl_common::FIELD_HEIGHT as f32 / 8.0 && !done {
            distance_reward
        } else {
            -1.0
        };
        */
        // Just survive!!!!
        let reward = if done {
            -1.0
        } else {
            0.1
        };
        info!("Reward: {}", reward);

        if client.send_obv(&state.renderer, reward, done).is_err() {
            handle_broken_socket();
        }
    }

    // Instantly reset the game on game over or level completion
    if state.training && done {
        state.cur_input = bulletrl_common::Input::empty();
        state.last_score = 0;
        state.frame = 0;
        (*(*GAME_UI).inner).show_results = 0;
        offsets::DESTROY_GAME_CHAINS();
        offsets::INIT_GAME_CHAINS();
        return 1;
    }

    // Wait for an input
    if let Some(client) = &mut state.client {
        if let Ok(input) = client.recv_input() {
            state.cur_input = input;
        } else {
            handle_broken_socket();
        }
    }

    1
}

// This is perfectly normal when exiting the train/eval script, so it's not really an error
fn handle_broken_socket() -> ! {
    info!("Socket broke, exiting!");
    std::process::abort();
}

unsafe fn render_observation(renderer: &mut bulletrl_common::Renderer) {
    // Draw order is important for readability
    // Also, some objects are drawn much larger than their actual hitbox to remain visible after downscaling
    renderer.clear();

    // Player
    renderer.draw_rect(
        0x00FF0000,
        (*PLAYER).pos.x as i32,
        (*PLAYER).pos.y as i32,
        25,
        25,
    );

    // Items
    for item in &(*ITEM_MANAGER).items {
        if item.active {
            renderer.draw_rect(0x00FFFF00, item.pos.x as i32, item.pos.y as i32, 16, 16);
        }
    }

    // Enemies
    for enemy in &(*ENEMY_MANAGER).enemies {
        if (enemy.enemy_type & 0x80) != 0 && (enemy.flags & 8) == 0 {
            renderer.draw_rect(
                0x0000FF00,
                enemy.pos.x as i32,
                enemy.pos.y as i32,
                enemy.size.x as i32,
                enemy.size.y as i32,
            );
        }
    }

    // Bullets
    for bullet in &(*ENEMY_BULLETS).bullets {
        if bullet.shot_type != 0 {
            renderer.draw_rect(
                0x000000FF,
                bullet.pos.x as i32,
                bullet.pos.y as i32,
                bullet.size.x as i32 * 2,
                bullet.size.y as i32 * 2,
            );
        }
    }

    // Lasers
    let rot = |mut point: Vector2, origin: Vector2, angle: f32| {
        point.x -= origin.x;
        point.y -= origin.y;

        let temp_x = point.x * angle.cos() - point.y * angle.sin();
        let temp_y = point.x * angle.sin() + point.y * angle.cos();

        Vector2::new(temp_x + origin.x, temp_y + origin.y)
    };

    for laser in &(*ENEMY_BULLETS).lasers {
        if laser.active != 0 {
            let pos = Vector2::new(
                (laser.end_offset - laser.start_offset) / 2.0
                    + laser.start_offset
                    + laser.fire_point.x,
                laser.fire_point.y,
            );
            let size = Vector2::new(laser.end_offset - laser.start_offset, laser.width / 2.0);

            let p1 = rot(
                Vector2::new(pos.x - (size.x / 2.0), pos.y),
                laser.fire_point,
                laser.angle,
            );
            let p2 = rot(
                Vector2::new(pos.x + (size.x / 2.0), pos.y),
                laser.fire_point,
                laser.angle,
            );

            renderer.draw_line(0x000000FF, p1, p2, size.y);
        }
    }
}

unsafe fn connect_to_server() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        warn!("No port specified, running in standalone non-training mode!");
        (*GLOBAL_STATE).training = false;
    } else if let Ok(port) = str::parse::<u16>(&args[1]) {
        let client = bulletrl_common::EnvClient::new(port).expect("connecting to server");
        (*GLOBAL_STATE).client = Some(client);
        (*GLOBAL_STATE).training = true; // TODO: Allow server to pick between eval and train
    } else {
        panic!("Failed to parse port: {}", args[1]);
    }
}

unsafe fn inject_calc_chain() {
    let node = offsets::CREATE_CHAIN_NODE(custom_calc);
    (*node).data = GLOBAL_STATE as *mut GlobalState as *mut c_void; // lol wtf
    offsets::ADD_CALC_CHAIN(offsets::CHAIN_MANAGER, node, 1000);
}

unsafe fn post_inject_init() {
    info!("Running post-injection initialization...");

    GLOBAL_STATE = Box::leak(Box::new(GlobalState {
        frame: 0,
        first_tick: true,
        renderer: Default::default(),
        client: None,
        training: false,
        cur_input: bulletrl_common::Input::empty(),
        last_score: 0,
        rng: rand::thread_rng(),

        #[cfg(feature = "renderer_debug")]
        window: minifb::Window::new(
            "bulletrl debug",
            bulletrl_common::FIELD_WIDTH,
            bulletrl_common::FIELD_HEIGHT,
            minifb::WindowOptions::default(),
        )
        .expect("creating debug window"),
    }));

    connect_to_server();
    if (*GLOBAL_STATE).training {
        patch::apply_training_patches();
        patch::install_training_hooks().expect("installing training hooks");
    }
    inject_calc_chain();

    info!("Initialization complete!");
}
