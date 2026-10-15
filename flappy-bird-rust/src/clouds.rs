use macroquad::prelude::*;
use crate::constants::*;

pub struct Cloud {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub speed: f32,
    pub alpha: f32,
}

impl Cloud {
    pub fn random(offscreen: bool) -> Self {
        let x = if offscreen {
            GAME_WIDTH + ::macroquad::rand::gen_range(0.0, 150.0)
        } else {
            ::macroquad::rand::gen_range(0.0, GAME_WIDTH)
        };
        Self {
            x,
            y: 30.0 + ::macroquad::rand::gen_range(0.0, GAME_HEIGHT * 0.35),
            w: 50.0 + ::macroquad::rand::gen_range(0.0, 60.0),
            h: 20.0 + ::macroquad::rand::gen_range(0.0, 20.0),
            speed: ::macroquad::rand::gen_range(CLOUD_MIN_SPEED, CLOUD_MAX_SPEED),
            alpha: ::macroquad::rand::gen_range(0.35, 0.7),
        }
    }
}

pub struct CloudManager {
    pub clouds: Vec<Cloud>,
}

impl CloudManager {
    pub fn new() -> Self {
        let mut clouds = Vec::new();
        for _ in 0..NUM_CLOUDS {
            clouds.push(Cloud::random(false));
        }
        Self { clouds }
    }

    pub fn update(&mut self, dt: f32) {
        for c in &mut self.clouds {
            c.x -= c.speed * dt;
            if c.x + c.w < -10.0 {
                *c = Cloud::random(true);
            }
        }
    }

    pub fn draw(&self) {
        for c in &self.clouds {
            let col = Color::new(1.0, 1.0, 1.0, c.alpha);
            // Main body
            draw_circle(c.x + c.w * 0.5, c.y + c.h * 0.5, c.h * 0.6, col);
            // Bumps
            draw_circle(c.x + c.w * 0.3, c.y + c.h * 0.4, c.h * 0.5, col);
            draw_circle(c.x + c.w * 0.7, c.y + c.h * 0.45, c.h * 0.55, col);
        }
    }
}
