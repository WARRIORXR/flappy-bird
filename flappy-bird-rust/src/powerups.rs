use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerUpType {
    Shield,      // Absorbs one collision
    Magnet,      // Pulls coins towards the bird for a duration
    SlowMo,      // Slows down pipe & obstacle speed for a duration
    DoubleScore, // Doubles points earned for a duration
}

impl PowerUpType {
    pub fn name(&self) -> &'static str {
        match self {
            PowerUpType::Shield => "SHIELD",
            PowerUpType::Magnet => "MAGNET",
            PowerUpType::SlowMo => "SLOW-MO",
            PowerUpType::DoubleScore => "2X SCORE",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            PowerUpType::Shield => Color::from_rgba(64, 224, 255, 255),      // Cyan
            PowerUpType::Magnet => Color::from_rgba(218, 112, 214, 255),     // Orchid/Purple
            PowerUpType::SlowMo => Color::from_rgba(255, 215, 0, 255),      // Gold
            PowerUpType::DoubleScore => Color::from_rgba(255, 69, 0, 255),  // OrangeRed
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            PowerUpType::Shield => "S",
            PowerUpType::Magnet => "M",
            PowerUpType::SlowMo => "T",
            PowerUpType::DoubleScore => "2",
        }
    }
}

#[derive(Clone)]
pub struct PowerUpItem {
    pub x: f32,
    pub y: f32,
    pub power_type: PowerUpType,
    pub collected: bool,
    pub anim_phase: f32,
}

pub struct PowerUpManager {
    pub items: Vec<PowerUpItem>,
    pub has_shield: bool,
    pub magnet_timer: f32,
    pub slowmo_timer: f32,
    pub invuln_timer: f32,        // Brief invulnerability after shield breaks
    pub double_score_timer: f32,  // Duration of 2x score multiplier
}

impl PowerUpManager {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            has_shield: false,
            magnet_timer: 0.0,
            slowmo_timer: 0.0,
            invuln_timer: 0.0,
            double_score_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.has_shield = false;
        self.magnet_timer = 0.0;
        self.slowmo_timer = 0.0;
        self.invuln_timer = 0.0;
        self.double_score_timer = 0.0;
    }

    pub fn spawn_for_pipe(&mut self, pipe_x: f32, gap_top: f32, gap_bottom: f32) {
        // 25% chance to spawn a power-up in a pipe
        if ::macroquad::rand::gen_range(0.0, 1.0) > 0.25 {
            return;
        }

        let types = [PowerUpType::Shield, PowerUpType::Magnet, PowerUpType::SlowMo, PowerUpType::DoubleScore];
        let chosen_type = types[::macroquad::rand::gen_range(0, types.len())];

        let center_y = (gap_top + gap_bottom) / 2.0;
        let y_offset = ::macroquad::rand::gen_range(-15.0, 15.0);

        self.items.push(PowerUpItem {
            x: pipe_x + PIPE_WIDTH / 2.0,
            y: center_y + y_offset,
            power_type: chosen_type,
            collected: false,
            anim_phase: ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU),
        });
    }

    pub fn update(&mut self, dt: f32, pipe_speed: f32, bird_x: f32, bird_y: f32) -> Option<PowerUpType> {
        // Decrease active timers
        if self.magnet_timer > 0.0 {
            self.magnet_timer = (self.magnet_timer - dt).max(0.0);
        }
        if self.slowmo_timer > 0.0 {
            self.slowmo_timer = (self.slowmo_timer - dt).max(0.0);
        }
        if self.invuln_timer > 0.0 {
            self.invuln_timer = (self.invuln_timer - dt).max(0.0);
        }
        if self.double_score_timer > 0.0 {
            self.double_score_timer = (self.double_score_timer - dt).max(0.0);
        }

        let mut collected_type = None;
        let bird_cx = bird_x + BIRD_WIDTH / 2.0;
        let bird_cy = bird_y + BIRD_HEIGHT / 2.0;
        let collect_radius = 24.0;

        for item in &mut self.items {
            item.x -= pipe_speed * dt;

            if item.collected {
                continue;
            }

            let dx = bird_cx - item.x;
            let dy = bird_cy - item.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < collect_radius {
                item.collected = true;
                collected_type = Some(item.power_type);

                match item.power_type {
                    PowerUpType::Shield => {
                        self.has_shield = true;
                    }
                    PowerUpType::Magnet => {
                        self.magnet_timer = 8.0;
                    }
                    PowerUpType::SlowMo => {
                        self.slowmo_timer = 6.0;
                    }
                    PowerUpType::DoubleScore => {
                        self.double_score_timer = 10.0;
                    }
                }
            }
        }

        self.items.retain(|item| item.x > -30.0 && !item.collected);
        collected_type
    }

    pub fn is_magnet_active(&self) -> bool {
        self.magnet_timer > 0.0
    }

    pub fn is_slowmo_active(&self) -> bool {
        self.slowmo_timer > 0.0
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invuln_timer > 0.0
    }

    pub fn is_double_score_active(&self) -> bool {
        self.double_score_timer > 0.0
    }

    pub fn pop_shield(&mut self) -> bool {
        if self.has_shield {
            self.has_shield = false;
            self.invuln_timer = 1.2; // 1.2s of mercy invulnerability
            true
        } else {
            false
        }
    }

    pub fn draw_items(&self, time: f32) {
        for item in &self.items {
            if item.collected {
                continue;
            }

            let bob_y = item.y + (time * 4.0 + item.anim_phase).sin() * 5.0;
            let base_color = item.power_type.color();

            // Outer glowing pulsing ring
            let pulse = 0.8 + (time * 6.0 + item.anim_phase).sin() * 0.2;
            let mut glow_col = base_color;
            glow_col.a = 0.4 * pulse;
            draw_circle(item.x, bob_y, 16.0 * pulse, glow_col);

            // Core circle
            draw_circle(item.x, bob_y, 11.0, base_color);
            draw_circle_lines(item.x, bob_y, 11.0, 1.8, WHITE);

            // Symbol text
            let sym = item.power_type.symbol();
            let dim = measure_text(sym, None, 14, 1.0);
            draw_text(sym, item.x - dim.width / 2.0, bob_y + 5.0, 14.0, BLACK);
        }
    }

    pub fn draw_bird_auras(&self, bird_x: f32, bird_y: f32, time: f32) {
        let cx = bird_x + BIRD_WIDTH / 2.0;
        let cy = bird_y + BIRD_HEIGHT / 2.0;

        // Magnet electric ring
        if self.is_magnet_active() {
            let pulse = 1.0 + (time * 8.0).sin() * 0.15;
            let mag_col = Color::from_rgba(218, 112, 214, 160);
            draw_poly_lines(cx, cy, 16, 26.0 * pulse, time * 120.0, 2.0, mag_col);
        }

        // Shield glowing bubble
        if self.has_shield {
            let pulse = 1.0 + (time * 5.0).sin() * 0.08;
            let shield_col = Color::from_rgba(64, 224, 255, 180);
            draw_poly_lines(cx, cy, 24, 24.0 * pulse, time * 60.0, 2.5, shield_col);
            let shield_fill = Color::from_rgba(64, 224, 255, 50);
            draw_circle(cx, cy, 24.0 * pulse, shield_fill);
        }

        // Double score fire aura
        if self.is_double_score_active() {
            let pulse = 1.0 + (time * 6.0).sin() * 0.12;
            let ds_col = Color::from_rgba(255, 69, 0, 140);
            draw_poly_lines(cx, cy, 6, 28.0 * pulse, time * 90.0, 2.0, ds_col);
        }
    }
}
