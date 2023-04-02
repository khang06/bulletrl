use std::ops::{Add, AddAssign};

#[derive(Default, Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

pub fn check_rect_overlap(p1: Vector2, s1: Vector2, p2: Vector2, s2: Vector2) -> bool {
    ((p1.x - p2.x).abs() * 2.0 < (s1.x + s2.x)) && ((p1.y - p2.y).abs() * 2.0 < (s1.y + s2.y))
}

pub fn ease_out_expo(start: f32, end: f32, t: f32) -> f32 {
    // https://easings.net/#easeOutExpo
    if t >= 1.0 {
        end
    } else {
        start + (1.0 - (2.0f32).powf(-10.0 * t)) * (end - start)
    }
}
