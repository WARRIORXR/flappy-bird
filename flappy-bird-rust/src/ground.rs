use macroquad::prelude::*;
use crate::constants::*;

pub struct Ground {
    pub offset: f32,
}

impl Ground {
    pub fn new() -> Self {
        Self { offset: 0.0 }
    }

    pub fn update(&mut self, dt: f32, speed: f32) {
        self.offset = (self.offset + speed * GROUND_SPEED_MULT * dt) % 24.0;
    }

    pub fn draw(&self) {
        let gy = GAME_HEIGHT - GROUND_HEIGHT;

        // Base dirt
        draw_rectangle(0.0, gy, GAME_WIDTH, GROUND_HEIGHT, Color::from_rgba(222, 184, 135, 255));

        // Grass strip
        let grass_h = 18.0;
        draw_rectangle(0.0, gy, GAME_WIDTH, grass_h, Color::from_rgba(76, 175, 80, 255));
        draw_rectangle(0.0, gy, GAME_WIDTH, 3.0, Color::from_rgba(102, 187, 106, 255));

        // Chevron pattern
        let count = (GAME_WIDTH / 24.0) as i32 + 2;
        for i in -1..count {
            let bx = (i as f32) * 24.0 - self.offset;
            let start = vec2(bx, gy + grass_h + 10.0);
            let mid = vec2(bx + 12.0, gy + grass_h + 22.0);
            let end = vec2(bx + 24.0, gy + grass_h + 10.0);
            draw_line(start.x, start.y, mid.x, mid.y, 1.5, Color::from_rgba(200, 169, 110, 255));
            draw_line(mid.x, mid.y, end.x, end.y, 1.5, Color::from_rgba(200, 169, 110, 255));
        }

        // Pebbles
        for i in 0..10 {
            let px = ((i as f32) * 40.0 + 15.0 - self.offset * 0.5).rem_euclid(GAME_WIDTH);
            draw_circle(px, gy + grass_h + 8.0, 2.5, Color::from_rgba(160, 137, 110, 255));
        }
    }
}
