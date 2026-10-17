use macroquad::prelude::*;
use crate::constants::*;

/// A collectible coin that floats in the gap between pipes.
#[derive(Clone)]
pub struct Coin {
    pub x: f32,
    pub y: f32,
    pub collected: bool,
    pub anim_offset: f32, // for bobbing animation
}

pub struct CoinManager {
    pub coins: Vec<Coin>,
    pub total_collected: u32,
}

impl CoinManager {
    pub fn new() -> Self {
        Self {
            coins: Vec::new(),
            total_collected: 0,
        }
    }

    pub fn reset(&mut self) {
        self.coins.clear();
        self.total_collected = 0;
    }

    /// Spawn a coin at the center of a pipe gap.
    pub fn spawn_for_pipe(&mut self, pipe_x: f32, gap_top: f32, gap_bottom: f32) {
        // Only spawn with 60% chance to keep it interesting
        if ::macroquad::rand::gen_range(0.0, 1.0) > 0.6 {
            return;
        }
        let center_y = (gap_top + gap_bottom) / 2.0;
        // Offset vertically a bit randomly within the gap
        let offset = ::macroquad::rand::gen_range(-20.0, 20.0);
        self.coins.push(Coin {
            x: pipe_x + PIPE_WIDTH / 2.0,
            y: center_y + offset,
            collected: false,
            anim_offset: ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU),
        });
    }

    /// Update coins: move with pipe speed, apply magnet attraction, check collection against bird.
    pub fn update(
        &mut self,
        dt: f32,
        pipe_speed: f32,
        bird_x: f32,
        bird_y: f32,
        magnet_active: bool,
    ) -> u32 {
        let mut collected_count = 0u32;
        let coin_radius = 10.0;
        let bird_cx = bird_x + BIRD_WIDTH / 2.0;
        let bird_cy = bird_y + BIRD_HEIGHT / 2.0;

        for coin in &mut self.coins {
            coin.x -= pipe_speed * dt;

            if coin.collected {
                continue;
            }

            // Magnetic attraction towards bird
            if magnet_active {
                let dx = bird_cx - coin.x;
                let dy = bird_cy - coin.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < 280.0 && dist > 1.0 {
                    let pull = (260.0 / dist).clamp(5.0, 14.0);
                    coin.x += (dx / dist) * pull * dt;
                    coin.y += (dy / dist) * pull * dt;
                }
            }

            // Overlap check for collection
            let dx = bird_cx - coin.x;
            let dy = bird_cy - coin.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < coin_radius + BIRD_WIDTH * 0.35 {
                coin.collected = true;
                collected_count += 1;
                self.total_collected += 1;
            }
        }

        // Remove coins that went off screen
        self.coins.retain(|c| c.x > -20.0 && !c.collected);

        collected_count
    }

    pub fn draw(&self, time: f32) {
        for coin in &self.coins {
            if coin.collected {
                continue;
            }

            let bob_y = coin.y + (time * 4.0 + coin.anim_offset).sin() * 4.0;
            let shimmer = 0.85 + (time * 6.0 + coin.anim_offset).sin() * 0.15;

            // Outer ring
            draw_circle(coin.x, bob_y, 10.0, Color::new(1.0 * shimmer, 0.84 * shimmer, 0.0, 1.0));
            draw_circle_lines(coin.x, bob_y, 10.0, 1.5, Color::from_rgba(180, 130, 0, 255));

            // Inner shine
            draw_circle(coin.x, bob_y, 6.0, Color::new(1.0, 0.95 * shimmer, 0.3, 1.0));

            // "$" symbol
            let c_dim = measure_text("$", None, 12, 1.0);
            draw_text("$", coin.x - c_dim.width / 2.0, bob_y + 4.0, 12.0, Color::from_rgba(140, 100, 0, 255));
        }
    }
}
