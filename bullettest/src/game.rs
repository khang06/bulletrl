use std::ops::RangeInclusive;

use crate::util::{self, check_rect_overlap, Vector2};
use bulletrl_common::{FIELD_HEIGHT, FIELD_WIDTH};
use log::info;
use rand::{
    distributions::Standard,
    prelude::{Distribution, ThreadRng},
    Rng,
};

const PLAYER_SIZE: i32 = 5;
const ENEMY_SIZE: i32 = 25;
const ENEMY_X_RANGE: RangeInclusive<f32> =
    (FIELD_WIDTH as f32 / 2.0) - 60.0..=(FIELD_WIDTH as f32 / 2.0) + 60.0;
const ENEMY_Y_RANGE: RangeInclusive<f32> = 50.0f32..=150.0f32;
const BULLET_LIMIT: usize = 640;

pub struct Game {
    pub renderer: bulletrl_common::Renderer,
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
    pub fn tick(&mut self, input: bulletrl_common::Input) -> bool {
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
                "Player got pichu~n'd, lasted {:.2} seconds...\n{:?} {:?}",
                self.frame as f64 / 60.0,
                self.enemy.movement,
                self.enemy.pattern
            );
            return true;
        }
        self.enemy.tick(&self.player, &mut self.bullets);

        self.draw();

        false
    }

    fn draw(&mut self) {
        self.renderer.clear();

        // Rendered from bottom to top
        // Order is important for visibility, especially at lower resolutions
        self.player.draw(&mut self.renderer);
        for x in self.bullets.iter_mut().flatten() {
            x.draw(&mut self.renderer);
        }
        self.enemy.draw(&mut self.renderer);
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
    pub fn tick(&mut self, input: bulletrl_common::Input, bullets: &mut [Option<Bullet>]) -> bool {
        use bulletrl_common::Input;

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

    pub fn draw(&self, renderer: &mut bulletrl_common::Renderer) {
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

#[derive(Clone, Copy, Debug)]
pub enum EnemyMovement {
    Static { pos: Vector2 },
    Sine { speed: f32, range: f32, height: f32 },
    EaseOutExpo { wait: u64, anim_len: u64 },
}

impl Distribution<EnemyMovement> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyMovement {
        match rng.gen_range(0..3) {
            0 => EnemyMovement::Static {
                pos: Vector2::new(rng.gen_range(ENEMY_X_RANGE), rng.gen_range(ENEMY_Y_RANGE)),
            },
            1 => EnemyMovement::Sine {
                speed: rng.gen_range(0.02f32..0.1f32),
                range: rng.gen_range(90.0..=125.0),
                height: rng.gen_range(ENEMY_Y_RANGE),
            },
            2 => EnemyMovement::EaseOutExpo {
                wait: rng.gen_range(30..=60),
                anim_len: rng.gen_range(30..=60),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EnemyPattern {
    Spiral {
        bullet_speed: f32,
        rot_speed: f32,
    },
    Direct {
        bullet_speed: f32,
        spread: f32,
        divisor: u64,
    },
    Burst {
        bullet_speed: f32,
        spread: f32,
        divisor: u64,
        amount: u64,
    },
}

impl Distribution<EnemyPattern> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyPattern {
        match rng.gen_range(0..3) {
            0 => EnemyPattern::Spiral {
                bullet_speed: rng.gen_range(2.0f32..5.0f32),
                rot_speed: rng.gen_range(0.1f32..1.0f32),
            },
            1 => EnemyPattern::Direct {
                bullet_speed: rng.gen_range(4.0f32..6.0f32),
                spread: rng.gen_range(0.0f32..2.0f32),
                divisor: rng.gen_range(4..=8),
            },
            2 => EnemyPattern::Burst {
                // This isn't actually very well designed
                // There are multiple scenarios where you can just sit and do nothing
                // Oh well
                bullet_speed: rng.gen_range(3.0f32..4.0f32),
                spread: rng.gen_range(std::f32::consts::FRAC_PI_4..std::f32::consts::FRAC_PI_2),
                divisor: rng.gen_range(6..=10),
                amount: rng.gen_range(4..=8),
            },
            _ => unreachable!(),
        }
    }
}

pub struct Enemy {
    pub pos: Vector2,
    pub target_pos: Vector2, // for EnemyMovement::EaseOutExpo
    pub last_pos: Vector2,   // for EnemyMovement::EaseOutExpo
    pub movement: EnemyMovement,
    pub pattern: EnemyPattern,
    pub frame: u64,
    rng: ThreadRng,
}

impl Default for Enemy {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Enemy {
            pos: Vector2::new(0.0, 0.0),
            target_pos: Vector2::new(rng.gen_range(ENEMY_X_RANGE), rng.gen_range(ENEMY_Y_RANGE)),
            last_pos: Vector2::new(rng.gen_range(ENEMY_X_RANGE), rng.gen_range(ENEMY_Y_RANGE)),
            movement: rng.gen(),
            pattern: rng.gen(),
            frame: 0,
            rng,
        }
    }
}

impl Enemy {
    pub fn tick(&mut self, player: &Player, bullets: &mut [Option<Bullet>]) {
        self.frame += 1;

        //self.pos.x = (FIELD_WIDTH as f32 / 2.0) + (self.frame as f64 / 25.0).sin() as f32 * 125.0;
        self.pos = match self.movement {
            EnemyMovement::Static { pos } => pos,
            EnemyMovement::Sine {
                speed,
                range,
                height,
            } => Vector2::new(
                (FIELD_WIDTH as f32 / 2.0) + (self.frame as f32 * speed).sin() as f32 * range,
                height,
            ),
            EnemyMovement::EaseOutExpo { wait, anim_len } => {
                let progress = self.frame % (anim_len + wait);
                let t = (progress as f32 / anim_len as f32).clamp(0.0, 1.0);
                if progress == 0 {
                    self.last_pos = self.target_pos;
                    self.target_pos = Vector2::new(
                        self.rng.gen_range(ENEMY_X_RANGE),
                        self.rng.gen_range(ENEMY_Y_RANGE),
                    );
                }

                Vector2::new(
                    util::ease_out_expo(self.last_pos.x, self.target_pos.x, t),
                    util::ease_out_expo(self.last_pos.y, self.target_pos.y, t),
                )
            }
        };

        match self.pattern {
            EnemyPattern::Spiral {
                bullet_speed,
                rot_speed,
            } => {
                let angle = self.frame as f32 * rot_speed;
                self.shoot(
                    bullets,
                    Vector2::new(
                        bullet_speed * angle.cos() as f32,
                        -bullet_speed * angle.sin() as f32,
                    ),
                );
            }
            EnemyPattern::Direct {
                bullet_speed,
                spread,
                divisor,
            } => {
                if self.frame % divisor == 0 {
                    let offset = if spread == 0.0 {
                        0.0
                    } else {
                        self.rng.gen_range(0.0f32..spread) - spread / 2.0
                    };
                    // https://stackoverflow.com/a/27481611
                    let delta_y = self.pos.y - player.pos.y;
                    let delta_x = player.pos.x - self.pos.x;
                    let angle = delta_y.atan2(delta_x) + offset;
                    self.shoot(
                        bullets,
                        Vector2::new(
                            bullet_speed * angle.cos() as f32,
                            -bullet_speed * angle.sin() as f32,
                        ),
                    );
                }
            }
            EnemyPattern::Burst {
                bullet_speed,
                spread,
                divisor,
                amount,
            } => {
                if self.frame % divisor == 0 {
                    // https://stackoverflow.com/a/27481611
                    let delta_y = self.pos.y - player.pos.y;
                    let delta_x = player.pos.x - self.pos.x;
                    let base_angle = delta_y.atan2(delta_x);

                    let spread_start = -spread / 2.0;
                    let spread_interval = spread / amount as f32;
                    for i in 0..amount {
                        let offset = spread_start + spread_interval * i as f32;
                        let angle = base_angle + offset;
                        self.shoot(
                            bullets,
                            Vector2::new(
                                bullet_speed * angle.cos() as f32,
                                -bullet_speed * angle.sin() as f32,
                            ),
                        );
                    }
                }
            }
        };
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

    pub fn draw(&mut self, renderer: &mut bulletrl_common::Renderer) {
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

    pub fn draw(&mut self, renderer: &mut bulletrl_common::Renderer) {
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
