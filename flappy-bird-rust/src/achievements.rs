use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Achievement {
    FirstFlight,     // Score 1 point
    Collector,       // Collect 10 coins in one game
    Survivor,        // Score 25 points
    ShieldMaster,    // Use 3 shields in one game
    SpeedDemon,      // Score 50 on Hard difficulty
    Perfectionist,   // 20 combo
    Centurion,       // Score 100 points
    CoinKing,        // Collect 50 coins total (lifetime)
    Veteran,         // Play 50 games
    WeatherWarrior,  // Score 30 in non-Clear weather
}

#[allow(dead_code)]
impl Achievement {
    pub fn name(&self) -> &'static str {
        match self {
            Achievement::FirstFlight => "First Flight",
            Achievement::Collector => "Coin Collector",
            Achievement::Survivor => "Survivor",
            Achievement::ShieldMaster => "Shield Master",
            Achievement::SpeedDemon => "Speed Demon",
            Achievement::Perfectionist => "Perfectionist",
            Achievement::Centurion => "Centurion",
            Achievement::CoinKing => "Coin King",
            Achievement::Veteran => "Veteran",
            Achievement::WeatherWarrior => "Weather Warrior",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Achievement::FirstFlight => "Score your first point",
            Achievement::Collector => "Collect 10 coins in one game",
            Achievement::Survivor => "Score 25 points",
            Achievement::ShieldMaster => "Use 3 shields in one game",
            Achievement::SpeedDemon => "Score 50 on Hard",
            Achievement::Perfectionist => "Get a 20x combo",
            Achievement::Centurion => "Score 100 points",
            Achievement::CoinKing => "Collect 50 lifetime coins",
            Achievement::Veteran => "Play 50 games",
            Achievement::WeatherWarrior => "Score 30 in weather",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Achievement::FirstFlight => "W",
            Achievement::Collector => "$",
            Achievement::Survivor => "S",
            Achievement::ShieldMaster => "D",
            Achievement::SpeedDemon => "F",
            Achievement::Perfectionist => "P",
            Achievement::Centurion => "C",
            Achievement::CoinKing => "K",
            Achievement::Veteran => "V",
            Achievement::WeatherWarrior => "R",
        }
    }

    pub fn all() -> &'static [Achievement] {
        &[
            Achievement::FirstFlight,
            Achievement::Collector,
            Achievement::Survivor,
            Achievement::ShieldMaster,
            Achievement::SpeedDemon,
            Achievement::Perfectionist,
            Achievement::Centurion,
            Achievement::CoinKing,
            Achievement::Veteran,
            Achievement::WeatherWarrior,
        ]
    }
}

pub struct AchievementManager {
    pub unlocked: Vec<Achievement>,
    pub notification: Option<(Achievement, f32)>, // (achievement, timer)
    pub shields_used: u32,  // per-game tracking
}

impl AchievementManager {
    pub fn new() -> Self {
        let unlocked = Self::load_unlocked();
        Self {
            unlocked,
            notification: None,
            shields_used: 0,
        }
    }

    pub fn reset_per_game(&mut self) {
        self.shields_used = 0;
    }

    fn load_unlocked() -> Vec<Achievement> {
        let contents = std::fs::read_to_string("achievements.txt").unwrap_or_default();
        let mut result = Vec::new();
        for line in contents.lines() {
            let trimmed = line.trim();
            for a in Achievement::all() {
                if a.name() == trimmed {
                    result.push(*a);
                    break;
                }
            }
        }
        result
    }

    fn save_unlocked(&self) {
        let data: String = self.unlocked.iter().map(|a| format!("{}\n", a.name())).collect();
        let _ = std::fs::write("achievements.txt", data);
    }

    pub fn try_unlock(&mut self, achievement: Achievement) -> bool {
        if self.unlocked.contains(&achievement) {
            return false;
        }
        self.unlocked.push(achievement);
        self.notification = Some((achievement, 3.0));
        self.save_unlocked();
        true
    }

    pub fn update(&mut self, dt: f32) {
        if let Some((_, ref mut timer)) = self.notification {
            *timer -= dt;
            if *timer <= 0.0 {
                self.notification = None;
            }
        }
    }

    pub fn draw_notification(&self) {
        if let Some((ref achievement, timer)) = self.notification {
            let alpha = (timer / 0.5).clamp(0.0, 1.0);
            let slide = if timer > 2.5 {
                1.0 - ((timer - 2.5) / 0.5) // slide in
            } else {
                1.0
            }.clamp(0.0, 1.0);
            let y = 85.0 + (1.0 - slide) * -40.0;

            // Background banner
            let bw = 220.0;
            let bh = 44.0;
            let bx = (GAME_WIDTH - bw) / 2.0;
            draw_rectangle(bx, y, bw, bh, Color::new(0.0, 0.0, 0.0, 0.7 * alpha));
            draw_rectangle_lines(bx, y, bw, bh, 2.0, Color::new(1.0, 0.84, 0.0, alpha));

            // Trophy icon
            draw_text("T", bx + 10.0, y + 28.0, 24.0, Color::new(1.0, 0.84, 0.0, alpha));

            // Achievement text
            let title = "ACHIEVEMENT UNLOCKED!";
            draw_text(title, bx + 36.0, y + 16.0, 11.0, Color::new(1.0, 0.84, 0.0, alpha));
            draw_text(achievement.name(), bx + 36.0, y + 34.0, 14.0, Color::new(1.0, 1.0, 1.0, alpha));
        }
    }

    pub fn draw_summary(&self) {
        // Draw on game over screen - show count of unlocked
        let total = Achievement::all().len();
        let unlocked = self.unlocked.len();
        let text = format!("Achievements: {}/{}", unlocked, total);
        let dim = measure_text(&text, None, 14, 1.0);
        draw_text(&text, (GAME_WIDTH - dim.width) / 2.0, 530.0, 14.0,
            Color::from_rgba(255, 215, 0, 200));
    }
}
