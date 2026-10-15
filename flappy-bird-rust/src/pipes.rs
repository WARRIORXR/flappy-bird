use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone)]
pub struct PipePair {
    pub x: f32,
    pub top_height: f32,
    pub bottom_y: f32,
    pub scored: bool,
    pub width: f32,
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

    pub fn reset(&mut self) {
        self.pipes.clear();
        for i in 0..4 {
            let x = GAME_WIDTH + 100.0 + (i as f32) * PIPE_SPACING;
            self.spawn_pipe(x);
        }
    }

    pub fn spawn_pipe(&mut self, x: f32) {
        let min_top = 60.0;
        let max_top = GAME_HEIGHT - GROUND_HEIGHT - self.gap - 60.0;
        let range = (max_top - min_top).max(10.0);
        let top_height = min_top + ::macroquad::rand::gen_range(0.0, range);

        self.pipes.push(PipePair {
            x,
            top_height,
            bottom_y: top_height + self.gap,
            scored: false,
            width: PIPE_WIDTH,
        });
    }

    pub fn get_speed(&self, score: u32) -> f32 {
        let base = (PIPE_SPEED + (score as f32) * PIPE_SPEED_INC).min(PIPE_MAX_SPEED);
        base * self.speed_mult
    }

    pub fn update(&mut self, dt: f32, score: &mut u32, bird_x: f32) -> bool {
        let speed = self.get_speed(*score);
        let mut scored_this_frame = false;

        for p in &mut self.pipes {
            p.x -= speed * dt;

            if !p.scored && p.x + p.width < bird_x {
                p.scored = true;
                *score += 1;
                scored_this_frame = true;
            }
        }

        self.pipes.retain(|p| p.x + p.width >= -20.0);

        if let Some(last) = self.pipes.last() {
            if last.x < GAME_WIDTH - PIPE_SPACING + PIPE_WIDTH {
                let next_x = last.x + PIPE_SPACING;
                self.spawn_pipe(next_x);
            }
        }

        scored_this_frame
    }

    pub fn draw(&self, time: f32) {
        let hue_shift = (time * 0.2).sin() * 0.05;
        let pipe_color = Color::new(0.24 + hue_shift, 0.75, 0.28, 1.0);
        let cap_color = Color::new(0.28 + hue_shift, 0.82, 0.32, 1.0);
        let border_color = Color::from_rgba(30, 107, 30, 255);

        for p in &self.pipes {
            // Top pipe
            draw_rectangle(p.x, 0.0, p.width, p.top_height, pipe_color);
            draw_rectangle_lines(p.x, 0.0, p.width, p.top_height, 2.0, border_color);

            let cap_w = p.width + 10.0;
            let cap_h = 24.0;
            let cap_x = p.x - 5.0;
            let cap_y_top = p.top_height - cap_h;
            draw_rectangle(cap_x, cap_y_top, cap_w, cap_h, cap_color);
            draw_rectangle_lines(cap_x, cap_y_top, cap_w, cap_h, 2.0, border_color);

            // Bottom pipe
            let bottom_h = GAME_HEIGHT - GROUND_HEIGHT - p.bottom_y;
            draw_rectangle(p.x, p.bottom_y, p.width, bottom_h, pipe_color);
            draw_rectangle_lines(p.x, p.bottom_y, p.width, bottom_h, 2.0, border_color);

            draw_rectangle(cap_x, p.bottom_y, cap_w, cap_h, cap_color);
            draw_rectangle_lines(cap_x, p.bottom_y, cap_w, cap_h, 2.0, border_color);
        }
    }
}
