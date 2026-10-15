use macroquad::prelude::*;
use crate::constants::*;

/// Available bird color skins.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BirdSkin {
    Classic,   // Yellow
    Blue,
    Red,
    Green,
    Purple,
}

impl BirdSkin {
    pub fn name(&self) -> &'static str {
        match self {
            BirdSkin::Classic => "CLASSIC",
            BirdSkin::Blue    => "BLUE",
            BirdSkin::Red     => "RED",
            BirdSkin::Green   => "GREEN",
            BirdSkin::Purple  => "PURPLE",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            BirdSkin::Classic => BirdSkin::Blue,
            BirdSkin::Blue    => BirdSkin::Red,
            BirdSkin::Red     => BirdSkin::Green,
            BirdSkin::Green   => BirdSkin::Purple,
            BirdSkin::Purple  => BirdSkin::Classic,
        }
    }

    /// Returns (body, belly, wing, outline) colors for this skin.
    pub fn colors(&self) -> (Color, Color, Color, Color) {
        match self {
            BirdSkin::Classic => (
                Color::from_rgba(255, 215, 0, 255),
                Color::from_rgba(255, 243, 176, 255),
                Color::from_rgba(240, 165, 0, 255),
                Color::from_rgba(218, 165, 32, 255),
            ),
            BirdSkin::Blue => (
                Color::from_rgba(70, 160, 255, 255),
                Color::from_rgba(170, 210, 255, 255),
                Color::from_rgba(40, 120, 220, 255),
                Color::from_rgba(30, 90, 180, 255),
            ),
            BirdSkin::Red => (
                Color::from_rgba(230, 70, 70, 255),
                Color::from_rgba(255, 170, 150, 255),
                Color::from_rgba(190, 40, 40, 255),
                Color::from_rgba(160, 30, 30, 255),
            ),
            BirdSkin::Green => (
                Color::from_rgba(80, 200, 80, 255),
                Color::from_rgba(170, 240, 170, 255),
                Color::from_rgba(50, 160, 50, 255),
                Color::from_rgba(30, 120, 30, 255),
            ),
            BirdSkin::Purple => (
                Color::from_rgba(170, 80, 220, 255),
                Color::from_rgba(210, 170, 255, 255),
                Color::from_rgba(130, 50, 180, 255),
                Color::from_rgba(100, 30, 150, 255),
            ),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bird {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub rotation: f32,
    pub wing_angle: f32,
    pub wing_dir: f32,
    pub flap_timer: f32,
    pub skin: BirdSkin,
    // Death animation fields
    pub dead: bool,
    pub death_vy: f32,
    pub death_spin: f32,
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
            skin: BirdSkin::Classic,
            dead: false,
            death_vy: 0.0,
            death_spin: 0.0,
        }
    }

    pub fn new_with_skin(skin: BirdSkin) -> Self {
        let mut b = Self::new();
        b.skin = skin;
        b
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

    /// Update during death animation — bird tumbles and falls off screen.
    pub fn update_death(&mut self, dt: f32) {
        self.death_vy += 0.35 * dt;
        self.y += self.death_vy * dt;
        self.death_spin += 0.15 * dt;
        self.rotation += self.death_spin;
    }

    /// Start death animation
    pub fn start_death(&mut self) {
        self.dead = true;
        self.death_vy = -6.0; // brief pop upward
        self.death_spin = 0.0;
    }

    /// Is the death animation done (bird fell below screen)?
    pub fn death_done(&self) -> bool {
        self.dead && self.y > GAME_HEIGHT + 50.0
    }

    pub fn draw(&self) {
        let (body_col, belly_col, wing_col, outline_col) = self.skin.colors();

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

        // Body
        draw_poly(center_x, center_y, 24, BIRD_WIDTH / 2.0, self.rotation.to_degrees(), body_col);
        draw_poly_lines(center_x, center_y, 24, BIRD_WIDTH / 2.0, self.rotation.to_degrees(), 1.5, outline_col);

        // Belly
        let belly_p = rotate_point(2.0, 3.0);
        draw_poly(belly_p.x, belly_p.y, 16, BIRD_WIDTH * 0.28, self.rotation.to_degrees(), belly_col);

        // Eye — X eyes if dead, normal otherwise
        let eye_p = rotate_point(BIRD_WIDTH * 0.22, -BIRD_HEIGHT * 0.15);
        if self.dead {
            // X-eyes
            let s = 4.0;
            draw_line(eye_p.x - s, eye_p.y - s, eye_p.x + s, eye_p.y + s, 2.0, BLACK);
            draw_line(eye_p.x + s, eye_p.y - s, eye_p.x - s, eye_p.y + s, 2.0, BLACK);
        } else {
            draw_circle(eye_p.x, eye_p.y, 6.0, WHITE);
            draw_circle_lines(eye_p.x, eye_p.y, 6.0, 1.0, Color::from_rgba(50, 50, 50, 255));
            let pupil_p = rotate_point(BIRD_WIDTH * 0.26, -BIRD_HEIGHT * 0.15);
            draw_circle(pupil_p.x, pupil_p.y, 3.0, BLACK);
        }

        // Beak
        let b1 = rotate_point(BIRD_WIDTH * 0.4, -2.0);
        let b2 = rotate_point(BIRD_WIDTH * 0.7, 2.0);
        let b3 = rotate_point(BIRD_WIDTH * 0.4, 6.0);
        draw_triangle(b1, b2, b3, Color::from_rgba(255, 102, 0, 255));

        // Wing
        let wing_y = 2.0 + self.wing_angle * 6.0;
        let wing_p = rotate_point(-4.0, wing_y);
        draw_poly(wing_p.x, wing_p.y, 16, 7.0, (self.rotation - 0.2).to_degrees(), wing_col);
        draw_poly_lines(wing_p.x, wing_p.y, 16, 7.0, (self.rotation - 0.2).to_degrees(), 1.0, outline_col);
    }
}
