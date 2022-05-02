use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::{error, info};
use minifb::{Key, Window, WindowOptions};

use crate::game::{self, Game, Input};

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
            game::FIELD_WIDTH,
            game::FIELD_HEIGHT,
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

            if self.game.tick(input) {
                self.game = Default::default();
            }

            self.window
                .update_with_buffer(
                    &self.game.renderer.buffer,
                    game::FIELD_WIDTH,
                    game::FIELD_HEIGHT,
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
    stream: TcpStream,
}

impl TcpBackend {
    pub fn new(port: u16) -> Self {
        info!("Connecting to port {}", port);
        let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port)))
            .expect("failed to connect");
        info!("Successfully connected!");

        TcpBackend {
            game: Default::default(),
            stream,
        }
    }
}

impl Backend for TcpBackend {
    fn main_loop(&mut self) {
        let mut input = Input::empty();
        let mut reset_pending = false;
        let mut died = false;
        let mut frame = 0;
        loop {
            // Play the game at 15fps to highlight major changes and for better performance
            let input_frame = frame % 4 == 0;
            frame += 1;

            // Read the input from the agent
            if input_frame {
                if let Ok(input_u8) = self.stream.read_u8() {
                    input = Input::from_bits_truncate(input_u8);
                } else {
                    break;
                }
            }

            // Avoid ticking the game until the next input frame if the player died or timed out
            if !reset_pending {
                died = self.game.tick(input);
            }
            let timeout = frame >= 60 * 60;
            reset_pending = died || timeout;

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
            if input_frame {
                if self
                    .stream
                    .write_all(bytemuck::cast_slice(&self.game.renderer.buffer))
                    .is_err()
                    || self.stream.write_f32::<LittleEndian>(reward).is_err()
                    || self.stream.write_u8((died || timeout) as u8).is_err()
                {
                    break;
                }
                if reset_pending {
                    if timeout {
                        info!("Player managed to survive 1 minute!");
                    }
                    self.game = Default::default();
                    reset_pending = false;
                    frame = 0;
                }
            }
        }
        error!("Stream broken, exiting!");
    }
}
