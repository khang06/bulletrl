use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::info;

pub const FIELD_WIDTH: usize = 384;
pub const FIELD_HEIGHT: usize = 448;

// Since I'm doing manual TCP communication, it's possible that it might desync due to a programming error
// This should catch that
const TCP_SENTINEL: u32 = 0x1337BEEF;

bitflags! {
    pub struct Input: u8 {
        const UP = 0b00000001;
        const DOWN = 0b00000010;
        const LEFT = 0b00000100;
        const RIGHT = 0b00001000;
        const FOCUS = 0b00010000;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub struct Renderer {
    pub buffer: Box<[u32]>,
    contours: Box<[i32]>,
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer {
            buffer: (vec![0; FIELD_WIDTH * FIELD_HEIGHT]).into_boxed_slice(),
            contours: (vec![-1; FIELD_HEIGHT * 2]).into_boxed_slice(),
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

    fn bresenham_update_contours(&mut self, p1: Vector2, p2: Vector2) {
        // http://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm#C
        let mut x0 = p1.x as i32;
        let mut y0 = p1.y as i32;
        let x1 = p2.x as i32;
        let y1 = p2.y as i32;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = (y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = (if dx > dy { dx } else { -dy }) / 2;

        loop {
            if y0 >= 0 && y0 < FIELD_HEIGHT as i32 {
                if x0 < self.contours[y0 as usize * 2] || self.contours[y0 as usize * 2] == -1 {
                    self.contours[y0 as usize * 2] = x0;
                }
                if x0 > self.contours[y0 as usize * 2 + 1]
                    || self.contours[y0 as usize * 2 + 1] == -1
                {
                    self.contours[y0 as usize * 2 + 1] = x0;
                }
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x0 += sx;
            }
            if e2 < dy {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_line(&mut self, color: u32, p1: Vector2, p2: Vector2, size: f32) {
        let perpendicular = (-((p2.x - p1.x) / (p2.y - p1.x))).atan();
        let psin = perpendicular.sin();
        let pcos = perpendicular.cos();

        let lp1 = Vector2::new(pcos * (-size / 2.0) + p1.x, psin * (-size / 2.0) + p1.y);
        let lp2 = Vector2::new(pcos * (size / 2.0) + p1.x, psin * (size / 2.0) + p1.y);
        let lp3 = Vector2::new(pcos * (size / 2.0) + p2.x, psin * (size / 2.0) + p2.y);
        let lp4 = Vector2::new(pcos * (-size / 2.0) + p2.x, psin * (-size / 2.0) + p2.y);

        self.contours.fill(-1);

        self.bresenham_update_contours(lp1, lp2);
        self.bresenham_update_contours(lp2, lp3);
        self.bresenham_update_contours(lp3, lp4);
        self.bresenham_update_contours(lp4, lp1);

        for y in 0..FIELD_HEIGHT {
            if self.contours[y * 2] != -1 && self.contours[y * 2 + 1] != -1 {
                for x in self.contours[y * 2]..=self.contours[y * 2 + 1] {
                    if x >= 0 && x < FIELD_WIDTH as i32 {
                        self.buffer[y * FIELD_WIDTH + x as usize] = color;
                    }
                }
            }
        }
    }
}

pub struct EnvClient {
    stream: TcpStream,
}

impl EnvClient {
    pub fn new(port: u16) -> Result<Self, std::io::Error> {
        info!("Connecting to port {}", port);
        let stream = TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port)))?;
        info!("Successfully connected!");

        Ok(EnvClient { stream })
    }

    pub fn recv_input(&mut self) -> Result<Input, std::io::Error> {
        assert_eq!(
            self.stream.read_u32::<LittleEndian>()?,
            TCP_SENTINEL,
            "TCP desync check failed!"
        );

        self.stream.read_u8().map(Input::from_bits_truncate)
    }

    pub fn send_obv(
        &mut self,
        renderer: &Renderer,
        reward: f32,
        done: bool,
    ) -> Result<(), std::io::Error> {
        self.stream.write_u32::<LittleEndian>(TCP_SENTINEL)?;
        self.stream
            .write_all(bytemuck::cast_slice(&renderer.buffer))?;
        self.stream.write_f32::<LittleEndian>(reward)?;
        self.stream.write_u8(done as u8)?;

        Ok(())
    }
}
