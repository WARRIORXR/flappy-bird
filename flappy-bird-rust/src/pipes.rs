use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone)]
pub struct PipePair {
    pub x: f32,
    pub top_height: f32,
    pub bottom_y: f32,
    pub scored: bool,
    pub width: f32,
    // Oscillation fields
    pub oscillates: bool,
    pub osc_center_top: f32,
    pub osc_phase: f32,
    pub osc_amplitude: f32,
}

pub struct PipeManager {
    pub pipes: Vec<PipePair>,
    pub gap: f32,
    pub speed_mult: f32,
}

impl PipeManager {
    pub fn new(gap: f32, speed_mult: f32) -> Self {
        let mut mgr = Self {
            pipes: Vec::new(),
            gap,
            speed_mult,
        };
        mgr.reset();
        mgr
    }

    pub fn reset(&mut self) -> Vec<(f32, f32, f32)> {
        self.pipes.clear();
        let mut gaps = Vec::new();
        for i in 0..4 {
            let x = GAME_WIDTH + 100.0 + (i as f32) * PIPE_SPACING;
            let gap = self.spawn_pipe(x, 0);
            gaps.push(gap);
        }
        gaps
    }

    pub fn spawn_pipe(&mut self, x: f32, score: u32) -> (f32, f32, f32) {
        let min_top = 60.0;
        let max_top = (GAME_HEIGHT - GROUND_HEIGHT - self.gap - 60.0).max(min_top + 10.0);
        let range = (max_top - min_top).max(10.0);
        let top_height = min_top + ::macroquad::rand::gen_range(0.0, range);

        // Pipes start oscillating after score 15, with increasing chance
        let osc_chance = if score > 15 {
            ((score - 15) as f32 * 0.03).min(0.6)
        } else {
            0.0
        };
        let oscillates = ::macroquad::rand::gen_range(0.0, 1.0) < osc_chance;
        let osc_amplitude = if oscillates {
            ::macroquad::rand::gen_range(15.0, 35.0)
        } else {
            0.0
        };

        let bottom_y = top_height + self.gap;

        self.pipes.push(PipePair {
            x,
            top_height,
            bottom_y,
            scored: false,
            width: PIPE_WIDTH,
            oscillates,
            osc_center_top: top_height,
            osc_phase: ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU),
            osc_amplitude,
        });

        (x, top_height, bottom_y)
    }

    pub fn get_speed(&self, score: u32) -> f32 {
        let base = (PIPE_SPEED + (score as f32) * PIPE_SPEED_INC).min(PIPE_MAX_SPEED);
        base * self.speed_mult
    }

    pub fn update(&mut self, dt: f32, score: &mut u32, bird_x: f32, time: f32) -> (bool, Option<(f32, f32, f32)>) {
        let speed = self.get_speed(*score);
        let mut scored_this_frame = false;
        let current_score = *score;

        for p in &mut self.pipes {
            p.x -= speed * dt;

            // Apply oscillation
            if p.oscillates {
                let osc = (time * 1.8 + p.osc_phase).sin() * p.osc_amplitude;
                // Clamp safely so pipes don't go off bounds
                let max_bound = (GAME_HEIGHT - GROUND_HEIGHT - self.gap - 50.0).max(50.0);
                let new_top = (p.osc_center_top + osc).clamp(50.0, max_bound);
                p.top_height = new_top;
                p.bottom_y = new_top + self.gap;
            }

            if !p.scored && p.x + p.width < bird_x {
                p.scored = true;
                *score += 1;
                scored_this_frame = true;
            }
        }

        self.pipes.retain(|p| p.x + p.width >= -20.0);

        let mut spawned_gap = None;
        if let Some(last) = self.pipes.last() {
            if last.x < GAME_WIDTH - PIPE_SPACING + PIPE_WIDTH {
                let next_x = last.x + PIPE_SPACING;
                let gap = self.spawn_pipe(next_x, current_score);
                spawned_gap = Some(gap);
            }
        }

        (scored_this_frame, spawned_gap)
    }

    pub fn draw(&self, time: f32) {
        let hue_shift = (time * 0.2).sin() * 0.05;
        let pipe_color = Color::new(0.24 + hue_shift, 0.75, 0.28, 1.0);
        let cap_color = Color::new(0.28 + hue_shift, 0.82, 0.32, 1.0);
        let border_color = Color::from_rgba(30, 107, 30, 255);
        // Moving pipes get a subtle red tint
        let moving_pipe_color = Color::new(0.28 + hue_shift, 0.65, 0.30, 1.0);
        let moving_cap_color = Color::new(0.32 + hue_shift, 0.72, 0.34, 1.0);

        for p in &self.pipes {
            let (pc, cc) = if p.oscillates {
                (moving_pipe_color, moving_cap_color)
            } else {
                (pipe_color, cap_color)
            };

            // Top pipe
            draw_rectangle(p.x, 0.0, p.width, p.top_height, pc);
            draw_rectangle_lines(p.x, 0.0, p.width, p.top_height, 2.0, border_color);

            let cap_w = p.width + 10.0;
            let cap_h = 24.0;
            let cap_x = p.x - 5.0;
            let cap_y_top = p.top_height - cap_h;
            draw_rectangle(cap_x, cap_y_top, cap_w, cap_h, cc);
            draw_rectangle_lines(cap_x, cap_y_top, cap_w, cap_h, 2.0, border_color);

            // Bottom pipe
            let bottom_h = GAME_HEIGHT - GROUND_HEIGHT - p.bottom_y;
            draw_rectangle(p.x, p.bottom_y, p.width, bottom_h, pc);
            draw_rectangle_lines(p.x, p.bottom_y, p.width, bottom_h, 2.0, border_color);

            draw_rectangle(cap_x, p.bottom_y, cap_w, cap_h, cc);
            draw_rectangle_lines(cap_x, p.bottom_y, cap_w, cap_h, 2.0, border_color);

            // Oscillation indicator arrows
            if p.oscillates {
                let arrow_x = p.x + p.width / 2.0;
                let arrow_y = p.top_height + self.gap / 2.0;
                let arrow_size = 5.0;
                // Up arrow
                draw_triangle(
                    vec2(arrow_x, arrow_y - 12.0),
                    vec2(arrow_x - arrow_size, arrow_y - 6.0),
                    vec2(arrow_x + arrow_size, arrow_y - 6.0),
                    Color::new(1.0, 1.0, 1.0, 0.4),
                );
                // Down arrow
                draw_triangle(
                    vec2(arrow_x, arrow_y + 12.0),
                    vec2(arrow_x - arrow_size, arrow_y + 6.0),
                    vec2(arrow_x + arrow_size, arrow_y + 6.0),
                    Color::new(1.0, 1.0, 1.0, 0.4),
                );
            }
        }
    }
}
