use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
        }
    }

    pub fn pipe_gap(&self) -> f32 {
        match self {
            Difficulty::Easy => 180.0,
            Difficulty::Normal => 150.0,
            Difficulty::Hard => 125.0,
        }
    }

    pub fn speed_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.85,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.25,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }
}

pub fn draw_start_screen(time: f32, difficulty: Difficulty) {
    let title = "Flappy Bird";
    let title_size = 46.0;
    let t_dim = measure_text(title, None, title_size as u16, 1.0);
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0 + 2.0, 162.0, title_size, Color::from_rgba(0, 0, 0, 70));
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0, 160.0, title_size, Color::from_rgba(255, 215, 0, 255));

    // Animated bird preview
    let preview_y = 310.0 + (time * 3.0).sin() * 15.0;
    let center_x = GAME_WIDTH / 2.0;
    draw_poly(center_x, preview_y, 24, 20.0, 0.0, Color::from_rgba(255, 215, 0, 255));
    draw_poly(center_x + 3.0, preview_y + 4.0, 16, 10.0, 0.0, Color::from_rgba(255, 243, 176, 255));
    draw_circle(center_x + 12.0, preview_y - 6.0, 6.0, WHITE);
    draw_circle(center_x + 14.0, preview_y - 6.0, 3.0, BLACK);
    draw_triangle(
        vec2(center_x + 18.0, preview_y - 2.0),
        vec2(center_x + 30.0, preview_y + 2.0),
        vec2(center_x + 18.0, preview_y + 6.0),
        Color::from_rgba(255, 102, 0, 255),
    );

    // Flashing start instruction
    let alpha = 0.5 + (time * 4.0).sin() * 0.45;
    let msg = "Tap / Space to Fly";
    let m_dim = measure_text(msg, None, 22, 1.0);
    draw_text(msg, (GAME_WIDTH - m_dim.width) / 2.0, 440.0, 22.0, Color::new(1.0, 1.0, 1.0, alpha));

    // Difficulty Selector
    let diff_text = format!("Difficulty: [{}] (Press D)", difficulty.name());
    let d_dim = measure_text(&diff_text, None, 18, 1.0);
    let diff_color = match difficulty {
        Difficulty::Easy => Color::from_rgba(100, 220, 100, 255),
        Difficulty::Normal => Color::from_rgba(255, 215, 0, 255),
        Difficulty::Hard => Color::from_rgba(255, 80, 80, 255),
    };
    draw_text(&diff_text, (GAME_WIDTH - d_dim.width) / 2.0, 490.0, 18.0, diff_color);

    // Controls hint
    let hint = "P: Pause | F: Toggle FPS";
    let h_dim = measure_text(hint, None, 15, 1.0);
    draw_text(hint, (GAME_WIDTH - h_dim.width) / 2.0, 530.0, 15.0, Color::from_rgba(200, 200, 200, 200));
}

pub fn draw_game_over(score: u32, best_score: u32) {
    // Dark veil
    draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, Color::from_rgba(0, 0, 0, 120));

    let title = "GAME OVER";
    let t_dim = measure_text(title, None, 44, 1.0);
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0 + 2.0, 182.0, 44.0, Color::from_rgba(0, 0, 0, 100));
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0, 180.0, 44.0, Color::from_rgba(255, 70, 70, 255));

    // Score Board
    let px = GAME_WIDTH / 2.0 - 120.0;
    let py = 230.0;
    let pw = 240.0;
    let ph = 170.0;

    draw_rectangle(px + 4.0, py + 4.0, pw, ph, Color::from_rgba(0, 0, 0, 60));
    draw_rectangle(px, py, pw, ph, Color::from_rgba(245, 222, 179, 255));
    draw_rectangle_lines(px, py, pw, ph, 3.0, Color::from_rgba(139, 115, 85, 255));

    // Score text
    let s_label = "SCORE";
    let sl_dim = measure_text(s_label, None, 18, 1.0);
    draw_text(s_label, (GAME_WIDTH - sl_dim.width) / 2.0, py + 32.0, 18.0, Color::from_rgba(100, 100, 100, 255));

    let s_str = score.to_string();
    let s_dim = measure_text(&s_str, None, 34, 1.0);
    draw_text(&s_str, (GAME_WIDTH - s_dim.width) / 2.0, py + 68.0, 34.0, Color::from_rgba(30, 30, 30, 255));

    // Best text
    let b_label = "BEST";
    let bl_dim = measure_text(b_label, None, 18, 1.0);
    draw_text(b_label, (GAME_WIDTH - bl_dim.width) / 2.0, py + 105.0, 18.0, Color::from_rgba(100, 100, 100, 255));

    let b_str = best_score.to_string();
    let b_dim = measure_text(&b_str, None, 28, 1.0);
    let b_col = if score >= best_score && score > 0 {
        Color::from_rgba(255, 180, 0, 255)
    } else {
        Color::from_rgba(50, 50, 50, 255)
    };
    draw_text(&b_str, (GAME_WIDTH - b_dim.width) / 2.0, py + 138.0, 28.0, b_col);

    // Medal
    let (medal_name, medal_col) = if score >= 50 {
        ("PLATINUM", Color::from_rgba(229, 228, 226, 255))
    } else if score >= 30 {
        ("GOLD", Color::from_rgba(255, 215, 0, 255))
    } else if score >= 20 {
        ("SILVER", Color::from_rgba(192, 192, 192, 255))
    } else if score >= 10 {
        ("BRONZE", Color::from_rgba(205, 127, 50, 255))
    } else {
        ("", WHITE)
    };

    if !medal_name.is_empty() {
        let mx = px + 45.0;
        let my = py + 80.0;
        draw_circle(mx, my, 18.0, medal_col);
        draw_circle_lines(mx, my, 18.0, 2.0, Color::from_rgba(60, 60, 60, 200));
        let m_dim = measure_text(medal_name, None, 11, 1.0);
        draw_text(medal_name, mx - m_dim.width / 2.0, my + 28.0, 11.0, Color::from_rgba(60, 60, 60, 255));
    }

    if score >= best_score && score > 0 {
        let nb = "★ NEW BEST! ★";
        let nb_dim = measure_text(nb, None, 18, 1.0);
        draw_text(nb, (GAME_WIDTH - nb_dim.width) / 2.0, py + ph + 28.0, 18.0, Color::from_rgba(255, 120, 0, 255));
    }

    // Restart button
    let bx = GAME_WIDTH / 2.0 - 75.0;
    let by = 470.0;
    let bw = 150.0;
    let bh = 46.0;

    draw_rectangle(bx + 3.0, by + 3.0, bw, bh, Color::from_rgba(0, 0, 0, 60));
    draw_rectangle(bx, by, bw, bh, Color::from_rgba(76, 175, 80, 255));
    draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::from_rgba(46, 125, 50, 255));

    let r_text = "RESTART";
    let r_dim = measure_text(r_text, None, 20, 1.0);
    draw_text(r_text, (GAME_WIDTH - r_dim.width) / 2.0, by + 29.0, 20.0, WHITE);
}

pub fn draw_hud(score: u32, fps: Option<i32>) {
    let s_str = score.to_string();
    let s_dim = measure_text(&s_str, None, 44, 1.0);
    let sx = (GAME_WIDTH - s_dim.width) / 2.0;
    draw_text(&s_str, sx + 2.0, 52.0, 44.0, Color::from_rgba(0, 0, 0, 120));
    draw_text(&s_str, sx, 50.0, 44.0, WHITE);

    if let Some(fps_val) = fps {
        let fps_str = format!("FPS: {}", fps_val);
        draw_text(&fps_str, 12.0, 24.0, 16.0, Color::from_rgba(50, 255, 50, 255));
    }
}
