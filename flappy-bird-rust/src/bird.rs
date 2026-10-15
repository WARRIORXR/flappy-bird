use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone, Copy)]
pub struct Bird {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub rotation: f32,
    pub wing_angle: f32,
    pub wing_dir: f32,
    pub flap_timer: f32,
}

impl Bird {
    pub fn new() -> Self {
        Self {
            x: BIRD_X,
            y: GAME_HEIGHT / 2.0 - 50.0,
            vy: 0.0,
            rotation: 0.0,
            wing_angle: 0.0,
            wing_dir: 1.0,
            flap_timer: 0.0,
        }
    }

    pub fn flap(&mut self) {
        self.vy = FLAP_STRENGTH;
        self.flap_timer = 8.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.vy += GRAVITY * dt;
        if self.vy > MAX_FALL_SPEED {
            self.vy = MAX_FALL_SPEED;
        }
        self.y += self.vy * dt;

        if self.vy < 0.0 {
            self.rotation = (self.rotation - 0.08 * dt).max(MAX_UP_ROTATION);
        } else {
            self.rotation = (self.rotation + 0.04 * dt).min(MAX_DOWN_ROTATION);
        }

        self.wing_angle += WING_FLAP_SPEED * self.wing_dir * dt;
        if self.wing_angle > 1.0 || self.wing_angle < -0.5 {
            self.wing_dir *= -1.0;
        }
        if self.flap_timer > 0.0 {
            self.flap_timer -= dt;
        }
    }

    pub fn draw(&self) {
        let center_x = self.x + BIRD_WIDTH / 2.0;
        let center_y = self.y + BIRD_HEIGHT / 2.0;
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        let rotate_point = |px: f32, py: f32| -> Vec2 {
            vec2(
                center_x + px * cos_r - py * sin_r,
                center_y + px * sin_r + py * cos_r,
            )
        };

        // Yellow body
        draw_poly(center_x, center_y, 24, BIRD_WIDTH / 2.0, self.rotation.to_degrees(), Color::from_rgba(255, 215, 0, 255));
        draw_poly_lines(center_x, center_y, 24, BIRD_WIDTH / 2.0, self.rotation.to_degrees(), 1.5, Color::from_rgba(218, 165, 32, 255));

        // Belly
        let belly_p = rotate_point(2.0, 3.0);
        draw_poly(belly_p.x, belly_p.y, 16, BIRD_WIDTH * 0.28, self.rotation.to_degrees(), Color::from_rgba(255, 243, 176, 255));

        // Eye
        let eye_p = rotate_point(BIRD_WIDTH * 0.22, -BIRD_HEIGHT * 0.15);
        draw_circle(eye_p.x, eye_p.y, 6.0, WHITE);
        draw_circle_lines(eye_p.x, eye_p.y, 6.0, 1.0, Color::from_rgba(50, 50, 50, 255));
        
        let pupil_p = rotate_point(BIRD_WIDTH * 0.26, -BIRD_HEIGHT * 0.15);
        draw_circle(pupil_p.x, pupil_p.y, 3.0, BLACK);

        // Beak
        let b1 = rotate_point(BIRD_WIDTH * 0.4, -2.0);
        let b2 = rotate_point(BIRD_WIDTH * 0.7, 2.0);
        let b3 = rotate_point(BIRD_WIDTH * 0.4, 6.0);
        draw_triangle(b1, b2, b3, Color::from_rgba(255, 102, 0, 255));

        // Wing
        let wing_y = 2.0 + self.wing_angle * 6.0;
        let wing_p = rotate_point(-4.0, wing_y);
        draw_poly(wing_p.x, wing_p.y, 16, 7.0, (self.rotation - 0.2).to_degrees(), Color::from_rgba(240, 165, 0, 255));
        draw_poly_lines(wing_p.x, wing_p.y, 16, 7.0, (self.rotation - 0.2).to_degrees(), 1.0, Color::from_rgba(218, 165, 32, 255));
    }
}
