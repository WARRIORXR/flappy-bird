use macroquad::prelude::*;
use crate::constants::*;
use crate::bird::BirdSkin;
use crate::weather::WeatherType;
use crate::powerups::PowerUpManager;
use crate::achievements::AchievementManager;

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

/// Milestone flash overlay — big text that fades out
pub struct ScoreFlash {
    pub text: String,
    pub timer: f32,
    pub max_timer: f32,
}

impl ScoreFlash {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            timer: 0.0,
            max_timer: 1.5,
        }
    }

    pub fn trigger(&mut self, text: &str) {
        self.text = text.to_string();
        self.timer = self.max_timer;
    }

    pub fn update(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer -= dt;
        }
    }

    pub fn draw(&self) {
        if self.timer <= 0.0 {
            return;
        }
        let alpha = (self.timer / self.max_timer).clamp(0.0, 1.0);
        let scale = 1.0 + (1.0 - alpha) * 0.5; // grows as it fades
        let font_size = (40.0 * scale) as u16;
        let dim = measure_text(&self.text, None, font_size, 1.0);
        let x = (GAME_WIDTH - dim.width) / 2.0;
        let y = GAME_HEIGHT * 0.35;

        // Glow shadow
        draw_text(&self.text, x + 2.0, y + 2.0, font_size as f32,
            Color::new(0.0, 0.0, 0.0, alpha * 0.5));
        // Main text
        draw_text(&self.text, x, y, font_size as f32,
            Color::new(1.0, 0.9, 0.2, alpha));
    }
}

/// Persistent game statistics
pub struct GameStats {
    pub games_played: u32,
    pub total_score: u32,
    pub total_coins: u32,
}

impl GameStats {
    pub fn load() -> Self {
        let contents = std::fs::read_to_string("stats.txt").unwrap_or_default();
        let mut games = 0u32;
        let mut total = 0u32;
        let mut coins = 0u32;
        for line in contents.lines() {
            if let Some(val) = line.strip_prefix("games=") {
                games = val.trim().parse().unwrap_or(0);
            } else if let Some(val) = line.strip_prefix("total_score=") {
                total = val.trim().parse().unwrap_or(0);
            } else if let Some(val) = line.strip_prefix("total_coins=") {
                coins = val.trim().parse().unwrap_or(0);
            }
        }
        Self { games_played: games, total_score: total, total_coins: coins }
    }

    pub fn save(&self) {
        let data = format!(
            "games={}\ntotal_score={}\ntotal_coins={}\n",
            self.games_played, self.total_score, self.total_coins
        );
        let _ = std::fs::write("stats.txt", data);
    }

    pub fn record_game(&mut self, score: u32, coins: u32) {
        self.games_played += 1;
        self.total_score += score;
        self.total_coins += coins;
        self.save();
    }
}

pub fn draw_start_screen(
    time: f32,
    difficulty: Difficulty,
    skin: BirdSkin,
    weather: WeatherType,
    muted: bool,
    stats: &GameStats,
    achievements: &AchievementManager,
) {
    let title = "Flappy Bird";
    let title_size = 46.0;
    let t_dim = measure_text(title, None, title_size as u16, 1.0);
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0 + 2.0, 132.0, title_size, Color::from_rgba(0, 0, 0, 70));
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0, 130.0, title_size, Color::from_rgba(255, 215, 0, 255));

    let sub = "— Rust Edition —";
    let sub_dim = measure_text(sub, None, 16, 1.0);
    draw_text(sub, (GAME_WIDTH - sub_dim.width) / 2.0, 155.0, 16.0, Color::from_rgba(200, 200, 200, 180));

    // Animated bird preview with selected skin
    let preview_y = 260.0 + (time * 3.0).sin() * 15.0;
    let center_x = GAME_WIDTH / 2.0;
    let (body, belly, _wing, outline) = skin.colors();
    draw_poly(center_x, preview_y, 24, 20.0, 0.0, body);
    draw_poly_lines(center_x, preview_y, 24, 20.0, 0.0, 1.5, outline);
    draw_poly(center_x + 3.0, preview_y + 4.0, 16, 10.0, 0.0, belly);
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
    draw_text(msg, (GAME_WIDTH - m_dim.width) / 2.0, 360.0, 22.0, Color::new(1.0, 1.0, 1.0, alpha));

    // Options Panel
    let opt_y = 405.0;

    // Difficulty Selector
    let diff_text = format!("Difficulty: [{}]  (D)", difficulty.name());
    let d_dim = measure_text(&diff_text, None, 16, 1.0);
    let diff_color = match difficulty {
        Difficulty::Easy => Color::from_rgba(100, 220, 100, 255),
        Difficulty::Normal => Color::from_rgba(255, 215, 0, 255),
        Difficulty::Hard => Color::from_rgba(255, 80, 80, 255),
    };
    draw_text(&diff_text, (GAME_WIDTH - d_dim.width) / 2.0, opt_y, 16.0, diff_color);

    // Bird Skin selector
    let skin_text = format!("Bird: [{}]  (C)", skin.name());
    let sk_dim = measure_text(&skin_text, None, 16, 1.0);
    let (sk_col, _, _, _) = skin.colors();
    draw_text(&skin_text, (GAME_WIDTH - sk_dim.width) / 2.0, opt_y + 24.0, 16.0, sk_col);

    // Weather selector
    let w_text = format!("Weather: [{}]  (W)", weather.name());
    let w_dim = measure_text(&w_text, None, 16, 1.0);
    draw_text(&w_text, (GAME_WIDTH - w_dim.width) / 2.0, opt_y + 48.0, 16.0, Color::from_rgba(160, 220, 255, 255));

    // Audio status
    let audio_label = if muted { "Audio: [MUTED]  (M)" } else { "Audio: [ON]  (M)" };
    let a_dim = measure_text(audio_label, None, 15, 1.0);
    let audio_color = if muted { Color::from_rgba(255, 120, 120, 255) } else { Color::from_rgba(180, 255, 180, 255) };
    draw_text(audio_label, (GAME_WIDTH - a_dim.width) / 2.0, opt_y + 70.0, 15.0, audio_color);

    // Controls hint
    let hint = "P: Pause | F: FPS | R: Restart";
    let h_dim = measure_text(hint, None, 13, 1.0);
    draw_text(hint, (GAME_WIDTH - h_dim.width) / 2.0, opt_y + 96.0, 13.0, Color::from_rgba(180, 180, 180, 190));

    // Stats box at bottom
    if stats.games_played > 0 {
        let stats_text = format!(
            "Games: {} | Total: {} | Coins: {}",
            stats.games_played, stats.total_score, stats.total_coins
        );
        let st_dim = measure_text(&stats_text, None, 13, 1.0);
        draw_text(&stats_text, (GAME_WIDTH - st_dim.width) / 2.0, 545.0, 13.0, Color::from_rgba(160, 160, 160, 180));
    }

    // Achievements progress
    let total_achievements = crate::achievements::Achievement::all().len();
    let unlocked_count = achievements.unlocked.len();
    if unlocked_count > 0 {
        let ach_text = format!("Achievements: {}/{}", unlocked_count, total_achievements);
        let ach_dim = measure_text(&ach_text, None, 13, 1.0);
        draw_text(&ach_text, (GAME_WIDTH - ach_dim.width) / 2.0, 565.0, 13.0, Color::from_rgba(255, 215, 0, 180));
    }
}

pub fn draw_game_over(score: u32, best_score: u32, coins_collected: u32) {
    // Dark veil
    draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, Color::from_rgba(0, 0, 0, 120));

    let title = "GAME OVER";
    let t_dim = measure_text(title, None, 44, 1.0);
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0 + 2.0, 162.0, 44.0, Color::from_rgba(0, 0, 0, 100));
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0, 160.0, 44.0, Color::from_rgba(255, 70, 70, 255));

    // Score Board
    let px = GAME_WIDTH / 2.0 - 120.0;
    let py = 200.0;
    let pw = 240.0;
    let ph = 210.0;

    draw_rectangle(px + 4.0, py + 4.0, pw, ph, Color::from_rgba(0, 0, 0, 60));
    draw_rectangle(px, py, pw, ph, Color::from_rgba(245, 222, 179, 255));
    draw_rectangle_lines(px, py, pw, ph, 3.0, Color::from_rgba(139, 115, 85, 255));

    // Score text
    let s_label = "SCORE";
    let sl_dim = measure_text(s_label, None, 18, 1.0);
    draw_text(s_label, (GAME_WIDTH - sl_dim.width) / 2.0, py + 32.0, 18.0, Color::from_rgba(100, 100, 100, 255));

    let s_str = score.to_string();
    let s_dim = measure_text(&s_str, None, 34, 1.0);
    draw_text(&s_str, (GAME_WIDTH - s_dim.width) / 2.0, py + 65.0, 34.0, Color::from_rgba(30, 30, 30, 255));

    // Coins collected
    let coin_str = format!("Coins: {}", coins_collected);
    let c_dim = measure_text(&coin_str, None, 16, 1.0);
    draw_text(&coin_str, (GAME_WIDTH - c_dim.width) / 2.0, py + 88.0, 16.0, Color::from_rgba(200, 160, 20, 255));

    // Divider
    draw_line(px + 20.0, py + 98.0, px + pw - 20.0, py + 98.0, 1.0, Color::from_rgba(200, 169, 110, 255));

    // Best text
    let b_label = "BEST";
    let bl_dim = measure_text(b_label, None, 18, 1.0);
    draw_text(b_label, (GAME_WIDTH - bl_dim.width) / 2.0, py + 120.0, 18.0, Color::from_rgba(100, 100, 100, 255));

    let b_str = best_score.to_string();
    let b_dim = measure_text(&b_str, None, 28, 1.0);
    let b_col = if score >= best_score && score > 0 {
        Color::from_rgba(255, 180, 0, 255)
    } else {
        Color::from_rgba(50, 50, 50, 255)
    };
    draw_text(&b_str, (GAME_WIDTH - b_dim.width) / 2.0, py + 148.0, 28.0, b_col);

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
        let mx = px + 40.0;
        let my = py + 70.0;
        draw_circle(mx, my, 18.0, medal_col);
        draw_circle_lines(mx, my, 18.0, 2.0, Color::from_rgba(60, 60, 60, 200));
        let star = "★";
        let star_dim = measure_text(star, None, 16, 1.0);
        draw_text(star, mx - star_dim.width / 2.0, my + 5.0, 16.0, Color::from_rgba(60, 60, 60, 220));
        let m_dim = measure_text(medal_name, None, 10, 1.0);
        draw_text(medal_name, mx - m_dim.width / 2.0, my + 28.0, 10.0, Color::from_rgba(60, 60, 60, 255));
    }

    if score >= best_score && score > 0 {
        let nb = "★ NEW BEST! ★";
        let nb_dim = measure_text(nb, None, 18, 1.0);
        draw_text(nb, (GAME_WIDTH - nb_dim.width) / 2.0, py + 178.0, 18.0, Color::from_rgba(255, 120, 0, 255));
    }

    // Restart button
    let bx = GAME_WIDTH / 2.0 - 75.0;
    let by = 450.0;
    let bw = 150.0;
    let bh = 46.0;

    draw_rectangle(bx + 3.0, by + 3.0, bw, bh, Color::from_rgba(0, 0, 0, 60));
    draw_rectangle(bx, by, bw, bh, Color::from_rgba(76, 175, 80, 255));
    draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::from_rgba(46, 125, 50, 255));

    let r_text = "RESTART";
    let r_dim = measure_text(r_text, None, 20, 1.0);
    draw_text(r_text, (GAME_WIDTH - r_dim.width) / 2.0, by + 29.0, 20.0, WHITE);

    let hint = "Space / Tap / R";
    let h_dim = measure_text(hint, None, 14, 1.0);
    draw_text(hint, (GAME_WIDTH - h_dim.width) / 2.0, by + bh + 22.0, 14.0, Color::from_rgba(180, 180, 180, 200));
}

pub fn draw_hud(score: u32, coins: u32, fps: Option<i32>, combo: u32, muted: bool, powerups: &PowerUpManager) {
    // Score
    let s_str = score.to_string();
    let s_dim = measure_text(&s_str, None, 44, 1.0);
    let sx = (GAME_WIDTH - s_dim.width) / 2.0;
    draw_text(&s_str, sx + 2.0, 52.0, 44.0, Color::from_rgba(0, 0, 0, 120));
    draw_text(&s_str, sx, 50.0, 44.0, WHITE);

    // Coin counter (top-left)
    let coin_str = format!("$ {}", coins);
    draw_text(&coin_str, 15.0, 50.0, 20.0, Color::from_rgba(255, 215, 0, 255));

    // Muted icon
    if muted {
        draw_text("MUTED", 15.0, 72.0, 13.0, Color::from_rgba(255, 100, 100, 220));
    }

    // Combo indicator (top-right area)
    if combo >= 3 {
        let combo_str = format!("x{} COMBO!", combo);
        let combo_col = if combo >= 10 {
            Color::from_rgba(255, 50, 50, 255)
        } else if combo >= 5 {
            Color::from_rgba(255, 165, 0, 255)
        } else {
            Color::from_rgba(100, 220, 255, 255)
        };
        let c_dim = measure_text(&combo_str, None, 18, 1.0);
        draw_text(&combo_str, GAME_WIDTH - c_dim.width - 12.0, 50.0, 18.0, combo_col);
    }

    // Active power-up indicators
    let mut badge_y = 75.0;
    if powerups.has_shield {
        let text = "[SHIELD]";
        let col = Color::from_rgba(64, 224, 255, 230);
        let dim = measure_text(text, None, 14, 1.0);
        draw_text(text, GAME_WIDTH - dim.width - 12.0, badge_y, 14.0, col);
        badge_y += 18.0;
    }
    if powerups.is_magnet_active() {
        let text = format!("[MAGNET {:.1}s]", powerups.magnet_timer);
        let col = Color::from_rgba(218, 112, 214, 230);
        let dim = measure_text(&text, None, 14, 1.0);
        draw_text(&text, GAME_WIDTH - dim.width - 12.0, badge_y, 14.0, col);
        badge_y += 18.0;
    }
    if powerups.is_slowmo_active() {
        let text = format!("[SLOW-MO {:.1}s]", powerups.slowmo_timer);
        let col = Color::from_rgba(255, 215, 0, 230);
        let dim = measure_text(&text, None, 14, 1.0);
        draw_text(&text, GAME_WIDTH - dim.width - 12.0, badge_y, 14.0, col);
        badge_y += 18.0;
    }
    if powerups.is_double_score_active() {
        let text = format!("[2X SCORE {:.1}s]", powerups.double_score_timer);
        let col = Color::from_rgba(255, 69, 0, 230);
        let dim = measure_text(&text, None, 14, 1.0);
        draw_text(&text, GAME_WIDTH - dim.width - 12.0, badge_y, 14.0, col);
        #[allow(unused_assignments)]
        { badge_y += 18.0; }
    }

    if let Some(fps_val) = fps {
        let fps_str = format!("FPS: {}", fps_val);
        draw_text(&fps_str, 12.0, 24.0, 16.0, Color::from_rgba(50, 255, 50, 255));
    }
}

/// Draw slow motion blue vignette edge effect
pub fn draw_slowmo_vignette() {
    let edge_col = Color::from_rgba(64, 224, 255, 60);
    // Draw top, bottom, left, right gradient/borders
    draw_rectangle(0.0, 0.0, GAME_WIDTH, 8.0, edge_col);
    draw_rectangle(0.0, GAME_HEIGHT - 8.0, GAME_WIDTH, 8.0, edge_col);
    draw_rectangle(0.0, 0.0, 8.0, GAME_HEIGHT, edge_col);
    draw_rectangle(GAME_WIDTH - 8.0, 0.0, 8.0, GAME_HEIGHT, edge_col);
}

/// Draw speed lines across the screen for dramatic high-speed effect
pub fn draw_speed_lines(time: f32, intensity: f32) {
    if intensity <= 0.0 {
        return;
    }
    let num_lines = (intensity * 12.0) as i32;
    let alpha = (intensity * 0.3).min(0.5);
    for i in 0..num_lines {
        let seed = (i as f32) * 137.5 + time * 80.0;
        let y = (seed * 7.3).sin() * GAME_HEIGHT * 0.5 + GAME_HEIGHT * 0.45;
        let len = 40.0 + (seed * 3.1).sin().abs() * 80.0;
        let x = ((seed * 1.7).sin() * 0.5 + 0.5) * GAME_WIDTH;
        draw_line(x, y, x + len, y, 1.5, Color::new(1.0, 1.0, 1.0, alpha));
    }
}
