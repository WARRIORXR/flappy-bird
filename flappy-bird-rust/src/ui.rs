use macroquad::prelude::*;
use crate::constants::*;
use crate::bird::BirdSkin;
use crate::weather::WeatherType;
use crate::powerups::PowerUpManager;
use crate::achievements::AchievementManager;
use crate::background::TimeOfDay;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    Solo,
    TwoPlayer,
}

impl GameMode {
    pub fn name(&self) -> &'static str {
        match self {
            GameMode::Solo => "SOLO",
            GameMode::TwoPlayer => "2 PLAYERS (DUEL)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            GameMode::Solo => GameMode::TwoPlayer,
            GameMode::TwoPlayer => GameMode::Solo,
        }
    }
}

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenFit {
    Fit,   // 4:7 aspect ratio with letterboxing
    Fill,  // Stretches to fill entire window (no black bars)
}

impl ScreenFit {
    pub fn name(&self) -> &'static str {
        match self {
            ScreenFit::Fit => "FIT (4:7)",
            ScreenFit::Fill => "FULL WINDOW",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            ScreenFit::Fit => ScreenFit::Fill,
            ScreenFit::Fill => ScreenFit::Fit,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WindowScale {
    Compact, // 400x700
    Medium,  // 480x840
    Large,   // 560x980
    Huge,    // 640x1120
}

impl WindowScale {
    pub fn size(&self) -> (f32, f32) {
        match self {
            WindowScale::Compact => (400.0, 700.0),
            WindowScale::Medium => (480.0, 840.0),
            WindowScale::Large => (560.0, 980.0),
            WindowScale::Huge => (640.0, 1120.0),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            WindowScale::Compact => "400x700",
            WindowScale::Medium => "480x840",
            WindowScale::Large => "560x980",
            WindowScale::Huge => "640x1120",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            WindowScale::Compact => WindowScale::Medium,
            WindowScale::Medium => WindowScale::Large,
            WindowScale::Large => WindowScale::Huge,
            WindowScale::Huge => WindowScale::Compact,
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

/// Floating score / bonus popup indicator
pub struct PopupText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: Color,
    pub font_size: f32,
    pub timer: f32,
    pub max_timer: f32,
    pub vy: f32,
}

pub struct PopupManager {
    pub popups: Vec<PopupText>,
}

impl PopupManager {
    pub fn new() -> Self {
        Self { popups: Vec::new() }
    }

    pub fn add(&mut self, text: impl Into<String>, x: f32, y: f32, color: Color, font_size: f32) {
        self.popups.push(PopupText {
            text: text.into(),
            x,
            y,
            color,
            font_size,
            timer: 1.0,
            max_timer: 1.0,
            vy: -48.0,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in self.popups.iter_mut() {
            p.timer -= dt;
            p.y += p.vy * dt;
        }
        self.popups.retain(|p| p.timer > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.popups {
            let alpha = (p.timer / p.max_timer).clamp(0.0, 1.0);
            let col = Color::new(p.color.r, p.color.g, p.color.b, p.color.a * alpha);
            let shadow_col = Color::new(0.0, 0.0, 0.0, 0.7 * alpha);
            let dim = measure_text(&p.text, None, p.font_size as u16, 1.0);
            let px = p.x - dim.width / 2.0;
            draw_text(&p.text, px + 1.5, p.y + 1.5, p.font_size, shadow_col);
            draw_text(&p.text, px, p.y, p.font_size, col);
        }
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

#[allow(clippy::too_many_arguments)]
pub fn draw_start_screen(
    time: f32,
    game_mode: GameMode,
    difficulty: Difficulty,
    skin: BirdSkin,
    weather: WeatherType,
    time_of_day: TimeOfDay,
    screen_fit: ScreenFit,
    window_scale: WindowScale,
    is_fullscreen: bool,
    muted: bool,
    stats: &GameStats,
    achievements: &AchievementManager,
) {
    let title = "Flappy Bird";
    let title_size = 46.0;
    let t_dim = measure_text(title, None, title_size as u16, 1.0);
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0 + 2.0, 126.0, title_size, Color::from_rgba(0, 0, 0, 70));
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0, 124.0, title_size, Color::from_rgba(255, 215, 0, 255));

    let sub = if game_mode == GameMode::TwoPlayer {
        "— 2 Players Duel Mode —"
    } else {
        "— Rust Edition —"
    };
    let sub_dim = measure_text(sub, None, 16, 1.0);
    let sub_color = if game_mode == GameMode::TwoPlayer {
        Color::from_rgba(100, 220, 255, 230)
    } else {
        Color::from_rgba(200, 200, 200, 180)
    };
    draw_text(sub, (GAME_WIDTH - sub_dim.width) / 2.0, 148.0, 16.0, sub_color);

    // Animated bird preview
    let preview_y = 230.0 + (time * 3.0).sin() * 10.0;
    let center_x = GAME_WIDTH / 2.0;

    if game_mode == GameMode::TwoPlayer {
        // Draw Bird 1 (P1)
        let p1_x = center_x - 55.0;
        let (b1, bel1, _, out1) = skin.colors();
        draw_poly(p1_x, preview_y, 24, 18.0, 0.0, b1);
        draw_poly_lines(p1_x, preview_y, 24, 18.0, 0.0, 1.5, out1);
        draw_poly(p1_x + 2.0, preview_y + 3.0, 16, 8.0, 0.0, bel1);
        draw_circle(p1_x + 10.0, preview_y - 5.0, 5.0, WHITE);
        draw_circle(p1_x + 12.0, preview_y - 5.0, 2.5, BLACK);
        draw_triangle(
            vec2(p1_x + 15.0, preview_y - 2.0),
            vec2(p1_x + 25.0, preview_y + 1.0),
            vec2(p1_x + 15.0, preview_y + 5.0),
            Color::from_rgba(255, 102, 0, 255),
        );
        let p1_label = "P1: Space/W";
        let p1_dim = measure_text(p1_label, None, 11, 1.0);
        draw_text(p1_label, p1_x - p1_dim.width / 2.0 + 5.0, preview_y + 30.0, 11.0, b1);

        // VS in center
        let vs = "VS";
        let vs_dim = measure_text(vs, None, 16, 1.0);
        draw_text(vs, center_x - vs_dim.width / 2.0, preview_y + 5.0, 16.0, Color::from_rgba(255, 215, 0, 220));

        // Draw Bird 2 (P2)
        let p2_x = center_x + 45.0;
        let p2_skin = if skin == BirdSkin::Blue { BirdSkin::Red } else { BirdSkin::Blue };
        let (b2, bel2, _, out2) = p2_skin.colors();
        draw_poly(p2_x, preview_y - (time * 3.0).cos() * 6.0, 24, 18.0, 0.0, b2);
        draw_poly_lines(p2_x, preview_y - (time * 3.0).cos() * 6.0, 24, 18.0, 0.0, 1.5, out2);
        draw_poly(p2_x + 2.0, preview_y + 3.0 - (time * 3.0).cos() * 6.0, 16, 8.0, 0.0, bel2);
        draw_circle(p2_x + 10.0, preview_y - 5.0 - (time * 3.0).cos() * 6.0, 5.0, WHITE);
        draw_circle(p2_x + 12.0, preview_y - 5.0 - (time * 3.0).cos() * 6.0, 2.5, BLACK);
        draw_triangle(
            vec2(p2_x + 15.0, preview_y - 2.0 - (time * 3.0).cos() * 6.0),
            vec2(p2_x + 25.0, preview_y + 1.0 - (time * 3.0).cos() * 6.0),
            vec2(p2_x + 15.0, preview_y + 5.0 - (time * 3.0).cos() * 6.0),
            Color::from_rgba(255, 102, 0, 255),
        );
        let p2_label = "P2: Up/K";
        let p2_dim = measure_text(p2_label, None, 11, 1.0);
        draw_text(p2_label, p2_x - p2_dim.width / 2.0 + 5.0, preview_y + 30.0, 11.0, b2);
    } else {
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
    }

    // Flashing start instruction
    let alpha = 0.5 + (time * 4.0).sin() * 0.45;
    let msg = if game_mode == GameMode::TwoPlayer {
        "Tap / Space to Start Duel"
    } else {
        "Tap / Space to Fly"
    };
    let m_dim = measure_text(msg, None, 22, 1.0);
    draw_text(msg, (GAME_WIDTH - m_dim.width) / 2.0, 312.0, 22.0, Color::new(1.0, 1.0, 1.0, alpha));

    // Options Panel
    let opt_y = 345.0;

    // Game Mode Selector
    let mode_text = format!("Mode: [{}]  (2 / Tab)", game_mode.name());
    let m_dim2 = measure_text(&mode_text, None, 14, 1.0);
    let mode_col = if game_mode == GameMode::TwoPlayer {
        Color::from_rgba(100, 220, 255, 255)
    } else {
        Color::from_rgba(255, 215, 0, 255)
    };
    draw_text(&mode_text, (GAME_WIDTH - m_dim2.width) / 2.0, opt_y, 14.0, mode_col);

    // Difficulty Selector
    let diff_text = format!("Difficulty: [{}]  (D)", difficulty.name());
    let d_dim = measure_text(&diff_text, None, 14, 1.0);
    let diff_color = match difficulty {
        Difficulty::Easy => Color::from_rgba(100, 220, 100, 255),
        Difficulty::Normal => Color::from_rgba(255, 215, 0, 255),
        Difficulty::Hard => Color::from_rgba(255, 80, 80, 255),
    };
    draw_text(&diff_text, (GAME_WIDTH - d_dim.width) / 2.0, opt_y + 19.0, 14.0, diff_color);

    // Bird Skin selector
    let skin_text = format!("Bird Skin: [{}]  (C)", skin.name());
    let sk_dim = measure_text(&skin_text, None, 14, 1.0);
    let (sk_col, _, _, _) = skin.colors();
    draw_text(&skin_text, (GAME_WIDTH - sk_dim.width) / 2.0, opt_y + 38.0, 14.0, sk_col);

    // Weather selector
    let w_text = format!("Weather: [{}]  (W)", weather.name());
    let w_dim = measure_text(&w_text, None, 14, 1.0);
    draw_text(&w_text, (GAME_WIDTH - w_dim.width) / 2.0, opt_y + 57.0, 14.0, Color::from_rgba(160, 220, 255, 255));

    // Time of Day selector
    let tod_text = format!("Sky: [{}]  (N)", time_of_day.name());
    let tod_dim = measure_text(&tod_text, None, 14, 1.0);
    let tod_col = match time_of_day {
        TimeOfDay::Auto => Color::from_rgba(180, 220, 255, 255),
        TimeOfDay::Day => Color::from_rgba(255, 230, 130, 255),
        TimeOfDay::Sunset => Color::from_rgba(255, 140, 80, 255),
        TimeOfDay::Night => Color::from_rgba(160, 170, 255, 255),
        TimeOfDay::Dawn => Color::from_rgba(255, 180, 200, 255),
    };
    draw_text(&tod_text, (GAME_WIDTH - tod_dim.width) / 2.0, opt_y + 76.0, 14.0, tod_col);

    // Screen Fit selector
    let fit_text = format!("Display: [{}]  (X)", screen_fit.name());
    let fit_dim = measure_text(&fit_text, None, 14, 1.0);
    let fit_col = match screen_fit {
        ScreenFit::Fill => Color::from_rgba(100, 255, 218, 255),
        ScreenFit::Fit => Color::from_rgba(220, 220, 120, 255),
    };
    draw_text(&fit_text, (GAME_WIDTH - fit_dim.width) / 2.0, opt_y + 95.0, 14.0, fit_col);

    // Window Size & Fullscreen selector
    let win_text = if is_fullscreen {
        "Screen: [FULLSCREEN]  (F11/Alt+Enter)".to_string()
    } else {
        format!("Window Size: [{}]  (Z) | F11", window_scale.name())
    };
    let win_dim = measure_text(&win_text, None, 14, 1.0);
    draw_text(&win_text, (GAME_WIDTH - win_dim.width) / 2.0, opt_y + 114.0, 14.0, Color::from_rgba(200, 240, 255, 255));

    // Audio status
    let audio_label = if muted { "Audio: [MUTED]  (M)" } else { "Audio: [ON]  (M)" };
    let a_dim = measure_text(audio_label, None, 14, 1.0);
    let audio_color = if muted { Color::from_rgba(255, 120, 120, 255) } else { Color::from_rgba(180, 255, 180, 255) };
    draw_text(audio_label, (GAME_WIDTH - a_dim.width) / 2.0, opt_y + 133.0, 14.0, audio_color);

    // Controls hint
    let hint = if game_mode == GameMode::TwoPlayer {
        "P1: Space/W | P2: Up/K | P: Pause | R: Restart"
    } else {
        "P: Pause | F: FPS | R: Restart | F11: Fullscreen"
    };
    let h_dim = measure_text(hint, None, 12, 1.0);
    draw_text(hint, (GAME_WIDTH - h_dim.width) / 2.0, opt_y + 155.0, 12.0, Color::from_rgba(180, 180, 180, 190));

    // Stats box at bottom
    if stats.games_played > 0 {
        let stats_text = format!(
            "Games: {} | Total: {} | Coins: {}",
            stats.games_played, stats.total_score, stats.total_coins
        );
        let st_dim = measure_text(&stats_text, None, 13, 1.0);
        draw_text(&stats_text, (GAME_WIDTH - st_dim.width) / 2.0, 550.0, 13.0, Color::from_rgba(160, 160, 160, 180));
    }

    // Achievements progress
    let total_achievements = crate::achievements::Achievement::all().len();
    let unlocked_count = achievements.unlocked.len();
    if unlocked_count > 0 {
        let ach_text = format!("Achievements: {}/{}", unlocked_count, total_achievements);
        let ach_dim = measure_text(&ach_text, None, 13, 1.0);
        draw_text(&ach_text, (GAME_WIDTH - ach_dim.width) / 2.0, 568.0, 13.0, Color::from_rgba(255, 215, 0, 180));
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

pub fn draw_hud_2p(
    score1: u32,
    score2: u32,
    alive1: bool,
    alive2: bool,
    skin1: BirdSkin,
    skin2: BirdSkin,
    coins: u32,
    muted: bool,
) {
    let (col1, _, _, _) = skin1.colors();
    let (col2, _, _, _) = skin2.colors();

    // P1 Badge (Top-Left)
    draw_rectangle(12.0, 16.0, 115.0, 48.0, Color::from_rgba(0, 0, 0, 160));
    draw_rectangle_lines(12.0, 16.0, 115.0, 48.0, 2.0, col1);
    let p1_title = if alive1 { "PLAYER 1" } else { "P1 (OUT)" };
    draw_text(p1_title, 20.0, 32.0, 13.0, if alive1 { col1 } else { Color::from_rgba(180, 180, 180, 200) });
    let s1_str = score1.to_string();
    draw_text(&s1_str, 20.0, 56.0, 24.0, WHITE);

    // VS in center
    let vs = "VS";
    let vs_dim = measure_text(vs, None, 22, 1.0);
    draw_text(vs, (GAME_WIDTH - vs_dim.width) / 2.0, 46.0, 22.0, Color::from_rgba(255, 215, 0, 230));

    // P2 Badge (Top-Right)
    let p2_w = 115.0;
    let p2_x = GAME_WIDTH - p2_w - 12.0;
    draw_rectangle(p2_x, 16.0, p2_w, 48.0, Color::from_rgba(0, 0, 0, 160));
    draw_rectangle_lines(p2_x, 16.0, p2_w, 48.0, 2.0, col2);
    let p2_title = if alive2 { "PLAYER 2" } else { "P2 (OUT)" };
    draw_text(p2_title, p2_x + 12.0, 32.0, 13.0, if alive2 { col2 } else { Color::from_rgba(180, 180, 180, 200) });
    let s2_str = score2.to_string();
    let s2_dim = measure_text(&s2_str, None, 24, 1.0);
    draw_text(&s2_str, p2_x + p2_w - s2_dim.width - 14.0, 56.0, 24.0, WHITE);

    // Coin & Mute indicators
    let coin_str = format!("$ {}", coins);
    draw_text(&coin_str, 16.0, 80.0, 15.0, Color::from_rgba(255, 215, 0, 255));
    if muted {
        let m_dim = measure_text("MUTED", None, 13, 1.0);
        draw_text("MUTED", GAME_WIDTH - m_dim.width - 16.0, 80.0, 13.0, Color::from_rgba(255, 100, 100, 220));
    }
}

pub fn draw_game_over_2p(
    score1: u32,
    score2: u32,
    skin1: BirdSkin,
    skin2: BirdSkin,
    coins_collected: u32,
) {
    // Dark veil
    draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, Color::from_rgba(0, 0, 0, 130));

    let title = "DUEL FINISHED!";
    let t_dim = measure_text(title, None, 38, 1.0);
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0 + 2.0, 142.0, 38.0, Color::from_rgba(0, 0, 0, 120));
    draw_text(title, (GAME_WIDTH - t_dim.width) / 2.0, 140.0, 38.0, Color::from_rgba(255, 215, 0, 255));

    // Winner Banner
    let (winner_text, winner_col) = if score1 > score2 {
        ("👑 PLAYER 1 WINS! 👑", Color::from_rgba(255, 215, 0, 255))
    } else if score2 > score1 {
        ("👑 PLAYER 2 WINS! 👑", Color::from_rgba(70, 160, 255, 255))
    } else {
        ("🤝 IT'S A DRAW! 🤝", Color::from_rgba(200, 200, 200, 255))
    };
    let w_dim = measure_text(winner_text, None, 22, 1.0);
    draw_text(winner_text, (GAME_WIDTH - w_dim.width) / 2.0, 178.0, 22.0, winner_col);

    // Duel Scorecard
    let px = GAME_WIDTH / 2.0 - 130.0;
    let py = 205.0;
    let pw = 260.0;
    let ph = 210.0;

    draw_rectangle(px + 4.0, py + 4.0, pw, ph, Color::from_rgba(0, 0, 0, 70));
    draw_rectangle(px, py, pw, ph, Color::from_rgba(245, 222, 179, 255));
    draw_rectangle_lines(px, py, pw, ph, 3.0, Color::from_rgba(139, 115, 85, 255));

    // Player 1 Column
    let (col1, _, _, _) = skin1.colors();
    draw_text("PLAYER 1", px + 24.0, py + 35.0, 16.0, col1);
    let s1_str = score1.to_string();
    let s1_dim = measure_text(&s1_str, None, 36, 1.0);
    draw_text(&s1_str, px + 24.0 + (65.0 - s1_dim.width) / 2.0, py + 80.0, 36.0, Color::from_rgba(30, 30, 30, 255));
    if score1 >= score2 && score1 > 0 {
        draw_text("★ WINNER", px + 22.0, py + 110.0, 13.0, Color::from_rgba(218, 165, 32, 255));
    }

    // Divider Line
    draw_line(px + pw / 2.0, py + 18.0, px + pw / 2.0, py + 140.0, 2.0, Color::from_rgba(180, 150, 100, 255));

    // Player 2 Column
    let (col2, _, _, _) = skin2.colors();
    draw_text("PLAYER 2", px + pw / 2.0 + 24.0, py + 35.0, 16.0, col2);
    let s2_str = score2.to_string();
    let s2_dim = measure_text(&s2_str, None, 36, 1.0);
    draw_text(&s2_str, px + pw / 2.0 + 24.0 + (65.0 - s2_dim.width) / 2.0, py + 80.0, 36.0, Color::from_rgba(30, 30, 30, 255));
    if score2 >= score1 && score2 > 0 {
        draw_text("★ WINNER", px + pw / 2.0 + 22.0, py + 110.0, 13.0, Color::from_rgba(218, 165, 32, 255));
    }

    // Bottom info in card
    draw_line(px + 15.0, py + 150.0, px + pw - 15.0, py + 150.0, 1.0, Color::from_rgba(180, 150, 100, 255));
    let coins_str = format!("Coins Collected: {}", coins_collected);
    let c_dim = measure_text(&coins_str, None, 14, 1.0);
    draw_text(&coins_str, (GAME_WIDTH - c_dim.width) / 2.0, py + 175.0, 14.0, Color::from_rgba(139, 100, 30, 255));
    let diff_info = "P1: Space/W | P2: Up/K";
    let di_dim = measure_text(diff_info, None, 12, 1.0);
    draw_text(diff_info, (GAME_WIDTH - di_dim.width) / 2.0, py + 195.0, 12.0, Color::from_rgba(100, 100, 100, 220));

    // Restart button
    let bx = GAME_WIDTH / 2.0 - 80.0;
    let by = 445.0;
    let bw = 160.0;
    let bh = 46.0;

    draw_rectangle(bx + 3.0, by + 3.0, bw, bh, Color::from_rgba(0, 0, 0, 60));
    draw_rectangle(bx, by, bw, bh, Color::from_rgba(76, 175, 80, 255));
    draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::from_rgba(46, 125, 50, 255));

    let r_text = "PLAY AGAIN";
    let r_dim = measure_text(r_text, None, 18, 1.0);
    draw_text(r_text, (GAME_WIDTH - r_dim.width) / 2.0, by + 28.0, 18.0, WHITE);

    let hint = "Space / Tap / R";
    let h_dim = measure_text(hint, None, 14, 1.0);
    draw_text(hint, (GAME_WIDTH - h_dim.width) / 2.0, by + bh + 22.0, 14.0, Color::from_rgba(180, 180, 180, 200));
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
