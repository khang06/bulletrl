use crate::util::{check_rect_overlap, Vector2};
use bitflags::bitflags;
use log::info;

pub const FIELD_WIDTH: usize = 384;
pub const FIELD_HEIGHT: usize = 448;
const PLAYER_SIZE: i32 = 5;
const ENEMY_SIZE: i32 = 25;
const BULLET_LIMIT: usize = 640;

bitflags! {
    pub struct Input: u8 {
        const UP = 0b00000001;
        const DOWN = 0b00000010;
        const LEFT = 0b00000100;
        const RIGHT = 0b00001000;
        const FOCUS = 0b00010000;
    }
}

pub struct Game {
    pub renderer: Renderer,
    pub player: Player,
    pub enemy: Enemy, // TODO: multiple enemies
    pub bullets: Box<[Option<Bullet>]>,
    pub frame: u64,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            renderer: Default::default(),
            player: Default::default(),
            enemy: Default::default(),
            bullets: (vec![None; BULLET_LIMIT]).into_boxed_slice(),
            frame: 0,
        }
    }
}

impl Game {
    pub fn tick(&mut self, input: Input) -> bool {
        self.frame += 1;
        for x in self.bullets.iter_mut() {
            if let Some(bullet) = x {
                if bullet.tick() {
                    *x = None;
                }
            }
        }
        if self.player.tick(input, &mut self.bullets) {
            info!(
                "Player got pichu~n'd, lasted {:.2} seconds",
                self.frame as f64 / 60.0
            );
            return true;
        }
        self.enemy.tick(&mut self.bullets);

        self.draw();

        false
    }

    fn draw(&mut self) {
        self.renderer.clear();

        // Rendered from bottom to top
        // Order is important for visibility, especialy at lower resolutions
        self.player.draw(&mut self.renderer);
        for x in self.bullets.iter_mut().flatten() {
            x.draw(&mut self.renderer);
        }
        self.enemy.draw(&mut self.renderer);
    }
}

pub struct Renderer {
    pub buffer: Box<[u32]>,
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer {
            buffer: (vec![0; FIELD_WIDTH * FIELD_HEIGHT]).into_boxed_slice(),
        }
    }
}

impl Renderer {
    pub fn clear(&mut self) {
        self.buffer.fill(0u32);
    }

    pub fn draw_rect(&mut self, color: u32, x: i32, y: i32, w: i32, h: i32) {
        let left = (x - w / 2).clamp(0, FIELD_WIDTH as i32 - 1) as usize;
        let right = (x + w / 2).clamp(0, FIELD_WIDTH as i32 - 1) as usize;
        let top = (y - h / 2).clamp(0, FIELD_HEIGHT as i32 - 1) as usize;
        let bottom = (y + h / 2).clamp(0, FIELD_HEIGHT as i32 - 1) as usize;

        for y in top..=bottom {
            for x in left..=right {
                self.buffer[y * FIELD_WIDTH + x] = color;
            }
        }
    }
}

pub struct Player {
    pub pos: Vector2,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            pos: Vector2::new(FIELD_WIDTH as f32 / 2.0, FIELD_HEIGHT as f32 - 50.0),
        }
    }
}

impl Player {
    pub fn tick(&mut self, input: Input, bullets: &mut [Option<Bullet>]) -> bool {
        // TODO: this kinda sucks
        let diagonal = input.contains(Input::UP | Input::LEFT)
            || input.contains(Input::UP | Input::RIGHT)
            || input.contains(Input::DOWN | Input::LEFT)
            || input.contains(Input::DOWN | Input::RIGHT);

        let mut speed = if input.contains(Input::FOCUS) {
            2.0f32
        } else {
            4.0f32
        };
        if diagonal {
            speed /= 2.0f32.sqrt();
        }

        if input.contains(Input::UP) {
            self.pos.y -= speed;
        }
        if input.contains(Input::DOWN) {
            self.pos.y += speed;
        }
        if input.contains(Input::LEFT) {
            self.pos.x -= speed;
        }
        if input.contains(Input::RIGHT) {
            self.pos.x += speed;
        }

        self.pos.x = self.pos.x.clamp(0.0, FIELD_WIDTH as f32);
        self.pos.y = self.pos.y.clamp(0.0, FIELD_HEIGHT as f32);

        for x in bullets.iter().flatten() {
            if check_rect_overlap(
                self.pos,
                Vector2::new(PLAYER_SIZE as f32, PLAYER_SIZE as f32),
                x.pos,
                x.size,
            ) {
                return true;
            }
        }
        false
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        // The hitbox is very small, so the player will be rendered larger to actually be visible
        renderer.draw_rect(
            0x00FF0000,
            self.pos.x as i32,
            self.pos.y as i32,
            PLAYER_SIZE * 5,
            PLAYER_SIZE * 5,
        );
    }
}

pub struct Enemy {
    pub pos: Vector2,
    pub frame: u64,
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy {
            pos: Vector2::new(FIELD_WIDTH as f32 / 2.0, 50.0),
            frame: 0,
        }
    }
}

impl Enemy {
    pub fn tick(&mut self, bullets: &mut [Option<Bullet>]) {
        self.frame += 1;

        self.pos.x = (FIELD_WIDTH as f32 / 2.0) + (self.frame as f64 / 25.0).sin() as f32 * 125.0;

        let angle = self.frame as f64 / 5.0;
        self.shoot(
            bullets,
            Vector2::new(-2.0 * angle.sin() as f32, 2.0 * angle.cos() as f32),
        );
    }

    fn shoot(&self, bullets: &mut [Option<Bullet>], velocity: Vector2) {
        // TODO: properly handle no free bullet slots!
        for x in bullets.iter_mut() {
            if x.is_none() {
                *x = Some(Bullet {
                    pos: self.pos,
                    size: Vector2::new(5.0, 5.0),
                    velocity,
                });
                return;
            }
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        renderer.draw_rect(
            0x0000FF00,
            self.pos.x as i32,
            self.pos.y as i32,
            ENEMY_SIZE,
            ENEMY_SIZE,
        );
    }
}

#[derive(Clone, Copy)]
pub struct Bullet {
    pub pos: Vector2,
    pub size: Vector2,
    pub velocity: Vector2,
}

impl Bullet {
    pub fn tick(&mut self) -> bool {
        if self.pos.x < 0.0 - self.size.x
            || self.pos.x > FIELD_WIDTH as f32 + self.size.x
            || self.pos.y < 0.0 - self.size.y
            || self.pos.y > FIELD_HEIGHT as f32 + self.size.y
        {
            return true;
        }

        self.pos += self.velocity;

        false
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        // Bullets are also drawn very large compared to their hitboxes, so they'll be scaled here too
        renderer.draw_rect(
            0x000000FF,
            self.pos.x as i32,
            self.pos.y as i32,
            self.size.x as i32 * 3,
            self.size.y as i32 * 3,
        );
    }
}
