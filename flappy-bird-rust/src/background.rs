use macroquad::prelude::*;
use crate::constants::*;

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub brightness: f32,
}

pub struct Background {
    pub stars: Vec<Star>,
}

impl Background {
    pub fn new() -> Self {
        let mut stars = Vec::new();
        for _ in 0..40 {
            stars.push(Star {
                x: ::macroquad::rand::gen_range(0.0, GAME_WIDTH),
                y: ::macroquad::rand::gen_range(0.0, GAME_HEIGHT * 0.6),
                size: ::macroquad::rand::gen_range(1.0, 2.5),
                brightness: ::macroquad::rand::gen_range(0.4, 1.0),
            });
        }
        Self { stars }
    }

    pub fn draw(&self, score: u32, time: f32) {
        // Cycle factor (0.0 to 1.0) repeats every 40 points
        let cycle = ((score % 40) as f32 / 40.0).clamp(0.0, 1.0);

        // Day -> Sunset -> Night transition
        let (top_color, bot_color, night_alpha) = if cycle < 0.35 {
            // Day
            (
                Color::from_rgba(135, 206, 235, 255),
                Color::from_rgba(224, 240, 255, 255),
                0.0,
            )
        } else if cycle < 0.65 {
            // Sunset
            (
                Color::from_rgba(240, 120, 80, 255),
                Color::from_rgba(255, 210, 140, 255),
                0.15,
            )
        } else {
            // Night
            (
                Color::from_rgba(18, 24, 58, 255),
                Color::from_rgba(40, 48, 88, 255),
                0.9,
            )
        };

        // Draw sky gradient in vertical bands
        let bands = 14;
        let band_h = GAME_HEIGHT / (bands as f32);
        for i in 0..bands {
            let t = (i as f32) / (bands as f32);
            let col = Color::new(
                top_color.r * (1.0 - t) + bot_color.r * t,
                top_color.g * (1.0 - t) + bot_color.g * t,
                top_color.b * (1.0 - t) + bot_color.b * t,
                1.0,
            );
            draw_rectangle(0.0, (i as f32) * band_h, GAME_WIDTH, band_h + 1.0, col);
        }

        // Stars appear during sunset/night
        if night_alpha > 0.05 {
            for s in &self.stars {
                let twinkle = 0.7 + (time * 3.0 + s.x).sin() * 0.3;
                let alpha = night_alpha * s.brightness * twinkle;
                draw_circle(s.x, s.y, s.size, Color::new(1.0, 1.0, 1.0, alpha));
            }
        }
    }
}
