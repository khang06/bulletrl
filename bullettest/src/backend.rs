use std::time::{Duration, Instant};

use bulletrl_common::Input;
use log::{error, info};
use minifb::{Key, Window, WindowOptions};

use crate::game::Game;

pub trait Backend {
    fn main_loop(&mut self);
}

pub struct MinifbBackend {
    game: Game,
    window: Window,
}

impl Default for MinifbBackend {
    fn default() -> Self {
        let mut window = Window::new(
            "bullettest",
            bulletrl_common::FIELD_WIDTH,
            bulletrl_common::FIELD_HEIGHT,
            //WindowOptions::default(),
            WindowOptions {
                topmost: true,
                ..Default::default()
            },
        )
        .expect("failed to create window");

        window.limit_update_rate(Some(Duration::from_secs_f64(1.0 / 60.0)));
        //window.limit_update_rate(None);

        MinifbBackend {
            game: Default::default(),
            window,
        }
    }
}

impl Backend for MinifbBackend {
    fn main_loop(&mut self) {
        let mut frames_in_sec = 0;
        let mut last_fps_update = Instant::now();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let mut input = Input::empty();
            if self.window.is_key_down(Key::Left) {
                input |= Input::LEFT;
            }
            if self.window.is_key_down(Key::Right) {
                input |= Input::RIGHT;
            }
            if self.window.is_key_down(Key::Up) {
                input |= Input::UP;
            }
            if self.window.is_key_down(Key::Down) {
                input |= Input::DOWN;
            }
            if self.window.is_key_down(Key::LeftShift) {
                input |= Input::FOCUS;
            }

            if self.game.tick(input) || self.window.is_key_down(Key::R) {
                self.game = Default::default();
            }

            self.window
                .update_with_buffer(
                    &self.game.renderer.buffer,
                    bulletrl_common::FIELD_WIDTH,
                    bulletrl_common::FIELD_HEIGHT,
                )
                .expect("failed to update window");

            if last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.window
                    .set_title(&format!("bullettest | {} fps", frames_in_sec));
                frames_in_sec = 0;
                last_fps_update = Instant::now();
            }
            frames_in_sec += 1;
        }
    }
}

pub struct TcpBackend {
    game: Game,
    client: bulletrl_common::EnvClient,
}

impl TcpBackend {
    pub fn new(port: u16) -> Self {
        let client = bulletrl_common::EnvClient::new(port).expect("connecting to server");
        TcpBackend {
            game: Default::default(),
            client,
        }
    }
}

impl Backend for TcpBackend {
    fn main_loop(&mut self) {
        let mut input = Input::empty();
        let mut frame = 0;
        loop {
            // Play the game at 15fps to highlight major changes and for better performance
            let input_frame = frame % 4 == 0;
            frame += 1;

            // Read the input from the agent
            if input_frame {
                if let Ok(input_recv) = self.client.recv_input() {
                    input = input_recv;
                } else {
                    break;
                }
            }

            let died = self.game.tick(input);
            let timeout = frame >= 60 * 60;

            // Send the current results to the agent
            let reward = if died {
                -1.0
            } else {
                // Reward being closer on the x-axis to the enemy
                1.0 - ((self.game.player.pos.x - self.game.enemy.pos.x)
                    .abs()
                    .clamp(0.0, 50.0)
                    / 50.0)
                    * 0.5
            };
            if died || timeout || input_frame {
                if self
                    .client
                    .send_obv(&self.game.renderer, reward, died || timeout)
                    .is_err()
                {
                    break;
                }
                if died || timeout {
                    if timeout {
                        info!(
                            "Player managed to survive 1 minute! {:?} {:?}",
                            self.game.enemy.movement, self.game.enemy.pattern
                        );
                    }
                    self.game = Default::default();
                    frame = 0;
                }
            }
        }
        error!("Stream broken, exiting!");
    }
}
