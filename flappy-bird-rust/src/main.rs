mod constants;
mod bird;
mod pipes;
mod ground;
mod clouds;
mod particles;
mod background;
mod audio;
mod ui;
mod coins;
mod powerups;
mod weather;
mod achievements;

use macroquad::prelude::*;
use constants::*;
use bird::{Bird, BirdSkin};
use pipes::PipeManager;
use ground::Ground;
use clouds::CloudManager;
use particles::ParticleSystem;
use background::Background;
use audio::AudioManager;
use coins::CoinManager;
use powerups::PowerUpManager;
use weather::{WeatherSystem, WeatherType};
use achievements::{AchievementManager, Achievement};
use ui::{
    Difficulty, GameMode, ScoreFlash, PopupManager, GameStats, ScreenFit, WindowScale,
    draw_start_screen, draw_game_over, draw_game_over_2p, draw_hud, draw_hud_2p, draw_speed_lines, draw_slowmo_vignette,
};

// ─── Game States ────────────────────────────────────────────────────────────
#[derive(PartialEq, Eq, Clone, Copy)]
enum GameState {
    Start,
    Playing,
    Dying,    // death animation before game over
    GameOver,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird - Rust Edition".to_string(),
        window_width: 400,
        window_height: 700,
        window_resizable: true,
        high_dpi: false,
        ..Default::default()
    }
}

fn load_high_score() -> u32 {
    std::fs::read_to_string("highscore.txt")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_high_score(score: u32) {
    let _ = std::fs::write("highscore.txt", score.to_string());
}

#[allow(unused_variables)]
fn save_score_history(score: u32, coins: u32, difficulty: Difficulty, skin: BirdSkin) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::Write;
        let diff_str = difficulty.name();
        let skin_str = skin.name();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry = format!(
            "{{\"score\":{},\"coins\":{},\"difficulty\":\"{}\",\"skin\":\"{}\",\"timestamp\":{}}}\n",
            score, coins, diff_str, skin_str, timestamp
        );
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("scores_history.jsonl") {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

fn check_collision(bird: &Bird, pipes: &PipeManager) -> bool {
    let margin = 3.0;
    let bx = bird.x + margin;
    let by = bird.y + margin;
    let bw = BIRD_WIDTH - margin * 2.0;
    let bh = BIRD_HEIGHT - margin * 2.0;

    // Generous top ceiling boundary to avoid unfair instant death on subtle brush
    if by <= -15.0 || by + bh >= GAME_HEIGHT - GROUND_HEIGHT {
        return true;
    }

    for p in &pipes.pipes {
        if bx + bw > p.x && bx < p.x + p.width && (by < p.top_height || by + bh > p.bottom_y) {
            return true;
        }
    }

    false
}

/// Check milestones for score flash
fn check_milestone(score: u32) -> Option<&'static str> {
    match score {
        10  => Some("NICE! 10!"),
        25  => Some("AMAZING! 25!"),
        50  => Some("LEGENDARY! 50!"),
        75  => Some("INSANE! 75!"),
        100 => Some("GOD MODE! 100!"),
        _   => None,
    }
}

/// Check milestones, achievements, and dynamic weather when score increases
fn check_score_events(
    score: u32,
    difficulty: Difficulty,
    weather: &mut WeatherSystem,
    achievements: &mut AchievementManager,
    score_flash: &mut ScoreFlash,
    combo: u32,
) {
    if let Some(msg) = check_milestone(score) {
        score_flash.trigger(msg);
    }

    if score >= 1 { achievements.try_unlock(Achievement::FirstFlight); }
    if score >= 25 { achievements.try_unlock(Achievement::Survivor); }
    if score >= 100 { achievements.try_unlock(Achievement::Centurion); }
    if score >= 50 && difficulty == Difficulty::Hard {
        achievements.try_unlock(Achievement::SpeedDemon);
    }
    if combo >= 20 {
        achievements.try_unlock(Achievement::Perfectionist);
    }
    if score >= 30 && weather.current != WeatherType::Clear {
        achievements.try_unlock(Achievement::WeatherWarrior);
    }

    // Automatic subtle weather transition at score milestones
    if score == 20 && weather.current == WeatherType::Clear {
        weather.set_weather(WeatherType::Rain);
    } else if score == 40 && weather.current == WeatherType::Rain {
        weather.set_weather(WeatherType::Snow);
    } else if score == 60 && weather.current == WeatherType::Snow {
        weather.set_weather(WeatherType::Sakura);
    } else if score == 80 && weather.current == WeatherType::Sakura {
        weather.set_weather(WeatherType::Firefly);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut audio = AudioManager::new().await;
    let mut bg = Background::new();
    let mut clouds = CloudManager::new();
    let mut ground = Ground::new();
    let mut particles = ParticleSystem::new();
    let mut weather = WeatherSystem::new();
    let mut achievements = AchievementManager::new();
    let mut popups = PopupManager::new();

    let mut game_mode = GameMode::Solo;
    let mut skin = BirdSkin::Classic;
    let mut skin2 = BirdSkin::Blue;
    let mut bird = Bird::new_with_skin(skin);
    let mut bird2 = Bird::new_with_skin_at(105.0, skin2);
    let mut score1 = 0u32;
    let mut score2 = 0u32;
    let mut alive1 = true;
    let mut alive2 = true;
    let mut duel_wins_1 = 0u32;
    let mut duel_wins_2 = 0u32;

    let mut difficulty = Difficulty::Normal;
    let mut pipes = PipeManager::new(difficulty.pipe_gap(), difficulty.speed_mult());
    let mut coin_mgr = CoinManager::new();
    let mut powerups = PowerUpManager::new();
    let mut score_flash = ScoreFlash::new();
    let mut stats = GameStats::load();

    let mut state = GameState::Start;
    let mut score = 0u32;
    let mut best_score = load_high_score();
    let mut paused = false;
    let mut show_fps = false;
    let mut combo = 0u32;        // consecutive pipes passed
    let mut coins_this_game = 0u32;
    let mut screen_shake = 0.0f32;
    let mut score_flash_white = 0.0f32; // brief white flash on score
    let mut game_time = 0.0f32;

    let mut screen_fit = ScreenFit::Fit;
    let mut window_scale = WindowScale::Compact;
    let mut is_fullscreen = false;

    loop {
        let raw_dt = get_frame_time();
        let dt = (raw_dt / 0.01667).min(3.0);
        game_time += raw_dt;
        score_flash.update(raw_dt);
        achievements.update(raw_dt);
        popups.update(raw_dt);
        bg.update(raw_dt);

        // ── Input: toggles ──────────────────────────────────────────────
        if is_key_pressed(KeyCode::F) {
            show_fps = !show_fps;
        }

        if is_key_pressed(KeyCode::M) {
            audio.toggle_mute();
        }

        if is_key_pressed(KeyCode::N) {
            let mode = bg.cycle_mode();
            if state == GameState::Playing {
                popups.add(format!("Sky: {}", mode.name()), GAME_WIDTH / 2.0, 100.0, Color::from_rgba(255, 230, 130, 255), 16.0);
            }
        }

        // Fullscreen toggle: F11 or Alt+Enter
        let alt_enter = (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt))
            && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter));
        if is_key_pressed(KeyCode::F11) || alt_enter {
            is_fullscreen = !is_fullscreen;
            macroquad::window::set_fullscreen(is_fullscreen);
            let msg = if is_fullscreen { "FULLSCREEN: ON" } else { "FULLSCREEN: OFF" };
            popups.add(msg, GAME_WIDTH / 2.0, 110.0, Color::from_rgba(100, 220, 255, 255), 18.0);
        }

        // Screen Fit toggle (X) - switches between Full Window and Fit (4:7)
        if is_key_pressed(KeyCode::X) {
            screen_fit = screen_fit.toggle();
            popups.add(
                format!("Display: {}", screen_fit.name()),
                GAME_WIDTH / 2.0,
                110.0,
                Color::from_rgba(100, 255, 218, 255),
                18.0,
            );
        }

        // Window size cycle (Z or F10) - cycles window scale presets
        if is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::F10) {
            if is_fullscreen {
                is_fullscreen = false;
                macroquad::window::set_fullscreen(false);
            }
            window_scale = window_scale.next();
            let (w, h) = window_scale.size();
            macroquad::window::request_new_screen_size(w, h);
            popups.add(
                format!("Window Size: {}", window_scale.name()),
                GAME_WIDTH / 2.0,
                110.0,
                Color::from_rgba(255, 220, 120, 255),
                18.0,
            );
        }

        if is_key_pressed(KeyCode::P) && state == GameState::Playing {
            paused = !paused;
        }

        // Start screen options
        if state == GameState::Start {
            if is_key_pressed(KeyCode::Key2) || is_key_pressed(KeyCode::Tab) {
                game_mode = game_mode.next();
            }
            if is_key_pressed(KeyCode::D) {
                difficulty = difficulty.next();
                pipes = PipeManager::new(difficulty.pipe_gap(), difficulty.speed_mult());
            }
            if is_key_pressed(KeyCode::C) {
                skin = skin.next();
                skin2 = if skin == BirdSkin::Blue { BirdSkin::Red } else { BirdSkin::Blue };
            }
            if is_key_pressed(KeyCode::W) {
                weather.toggle_weather();
            }
        }

        let (p1_flap, p2_flap) = match game_mode {
            GameMode::Solo => {
                let has_touch = touches().iter().any(|t| t.phase == TouchPhase::Started);
                let flap = is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Up)
                    || is_key_pressed(KeyCode::W)
                    || is_mouse_button_pressed(MouseButton::Left)
                    || has_touch;
                (flap, false)
            }
            GameMode::TwoPlayer => {
                let touch_left = touches().iter().any(|t| t.phase == TouchPhase::Started && t.position.x < screen_width() / 2.0);
                let touch_right = touches().iter().any(|t| t.phase == TouchPhase::Started && t.position.x >= screen_width() / 2.0);
                let mouse_left = is_mouse_button_pressed(MouseButton::Left) && mouse_position().0 < screen_width() / 2.0;
                let mouse_right = (is_mouse_button_pressed(MouseButton::Left) && mouse_position().0 >= screen_width() / 2.0)
                    || is_mouse_button_pressed(MouseButton::Right);

                let p1 = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::W) || touch_left || mouse_left;
                let p2 = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::K) || touch_right || mouse_right;
                (p1, p2)
            }
        };
        let tap = p1_flap || p2_flap;

        // ── State machine ───────────────────────────────────────────────
        match state {
            GameState::Start => {
                clouds.update(dt);
                weather.update(dt);

                if tap {
                    state = GameState::Playing;
                    score = 0;
                    score1 = 0;
                    score2 = 0;
                    alive1 = true;
                    alive2 = true;
                    combo = 0;
                    coins_this_game = 0;
                    bird = if game_mode == GameMode::TwoPlayer {
                        Bird::new_with_skin_at(75.0, skin)
                    } else {
                        Bird::new_with_skin_at(BIRD_X, skin)
                    };
                    skin2 = if skin == BirdSkin::Blue { BirdSkin::Red } else { BirdSkin::Blue };
                    bird2 = Bird::new_with_skin_at(105.0, skin2);
                    pipes = PipeManager::new(difficulty.pipe_gap(), difficulty.speed_mult());
                    let initial_gaps = pipes.reset();
                    coin_mgr.reset();
                    powerups.reset();
                    achievements.reset_per_game();
                    for gap in initial_gaps.iter().skip(1) {
                        coin_mgr.spawn_for_pipe(gap.0, gap.1, gap.2);
                        powerups.spawn_for_pipe(gap.0, gap.1, gap.2);
                    }
                    bird.flap();
                    if game_mode == GameMode::TwoPlayer {
                        bird2.flap();
                    }
                    audio.play_flap();
                }
            }

            GameState::Playing => {
                if !paused {
                    // Slow motion affects pipe and ground scroll speed
                    let slowmo_factor = if powerups.is_slowmo_active() { 0.6 } else { 1.0 };
                    let effective_dt = dt * slowmo_factor;

                    if alive1 {
                        bird.update(dt);
                        particles.add_trail(bird.x + 5.0, bird.y + BIRD_HEIGHT / 2.0);
                    } else {
                        bird.update_death(dt);
                    }

                    if game_mode == GameMode::TwoPlayer {
                        if alive2 {
                            bird2.update(dt);
                            particles.add_trail(bird2.x + 5.0, bird2.y + BIRD_HEIGHT / 2.0);
                        } else {
                            bird2.update_death(dt);
                        }
                    }

                    if p1_flap && alive1 {
                        bird.flap();
                        audio.play_flap();
                    }

                    if p2_flap && game_mode == GameMode::TwoPlayer && alive2 {
                        bird2.flap();
                        audio.play_flap_2();
                    }

                    // Calculate score multiplier based on active power-ups and difficulty
                    let score_multiplier: u32 = {
                        let mut mult = 1u32;
                        if powerups.is_double_score_active() { mult *= 2; }
                        if difficulty == Difficulty::Hard { mult += 1; }
                        mult
                    };

                    let spawned_gap = if game_mode == GameMode::TwoPlayer {
                        let (ev1, ev2, gap) = pipes.update_2p(
                            effective_dt,
                            &mut score1,
                            &mut score2,
                            bird.x,
                            alive1,
                            bird2.x,
                            alive2,
                            game_time,
                        );
                        if let Some(ev) = ev1 {
                            audio.play_score();
                            particles.add_sparkle(bird.x + BIRD_WIDTH + 8.0, bird.y);
                            popups.add(format!("P1 +{}", ev.points), bird.x + 10.0, bird.y - 15.0, Color::from_rgba(255, 215, 0, 255), 14.0);
                        }
                        if let Some(ev) = ev2 {
                            audio.play_score();
                            particles.add_sparkle(bird2.x + BIRD_WIDTH + 8.0, bird2.y);
                            popups.add(format!("P2 +{}", ev.points), bird2.x + 10.0, bird2.y - 15.0, Color::from_rgba(70, 160, 255, 255), 14.0);
                        }
                        score = score1.max(score2);
                        gap
                    } else {
                        let (pipe_score_ev, gap) = pipes.update(effective_dt, &mut score, bird.x, game_time);
                        if let Some(ev) = pipe_score_ev {
                            // Apply score multiplier (ev.points is already added once by pipes.update)
                            if score_multiplier > 1 {
                                score += ev.points * (score_multiplier - 1);
                            }

                            let total_pts = ev.points * score_multiplier;
                            combo += 1;

                            if ev.is_golden {
                                audio.play_golden();
                                popups.add(
                                    format!("+{} GOLDEN PIPE!", total_pts),
                                    ev.x,
                                    ev.y,
                                    Color::from_rgba(255, 215, 0, 255),
                                    18.0,
                                );
                                score_flash.trigger("GOLDEN PIPE! +5");
                                particles.add_powerup_burst(ev.x, ev.y, Color::from_rgba(255, 215, 0, 255));
                                score_flash_white = 0.25;
                            } else {
                                audio.play_score();
                                popups.add(
                                    format!("+{}", total_pts),
                                    ev.x,
                                    ev.y,
                                    WHITE,
                                    16.0,
                                );
                                particles.add_sparkle(bird.x + BIRD_WIDTH + 10.0, bird.y);
                                score_flash_white = 0.15;
                            }

                            check_score_events(score, difficulty, &mut weather, &mut achievements, &mut score_flash, combo);
                        }
                        gap
                    };

                    // Spawn coins & powerups when new pipes appear
                    if let Some((px, top_h, bot_y)) = spawned_gap {
                        coin_mgr.spawn_for_pipe(px, top_h, bot_y);
                        powerups.spawn_for_pipe(px, top_h, bot_y);
                    }

                    // Check for Close Shave near-miss bonus (+2 points) in Solo mode
                    if game_mode == GameMode::Solo {
                        if let Some((sx, sy)) = pipes.check_close_shave(bird.x, bird.y) {
                            let shave_pts = 2 * score_multiplier;
                            score += shave_pts;
                            audio.play_close_shave();
                            particles.add_sparkle(sx, sy);
                            particles.add_sparkle(sx + 5.0, sy - 5.0);
                            popups.add(
                                format!("+{} CLOSE SHAVE!", shave_pts),
                                sx,
                                sy - 15.0,
                                Color::from_rgba(64, 224, 255, 255),
                                16.0,
                            );
                            check_score_events(score, difficulty, &mut weather, &mut achievements, &mut score_flash, combo);
                        }
                    }

                    // Update powerups (effective_dt for movement, raw_dt for ability timers)
                    let pipe_speed = pipes.get_speed(score) * slowmo_factor;
                    if let Some(pup) = powerups.update(effective_dt, raw_dt, pipe_speed, bird.x, bird.y) {
                        audio.play_powerup();
                        particles.add_powerup_burst(
                            bird.x + BIRD_WIDTH / 2.0,
                            bird.y + BIRD_HEIGHT / 2.0,
                            pup.color(),
                        );
                        score_flash.trigger(pup.name());
                    }

                    // Update coins with magnet attraction
                    let coins_got = coin_mgr.update(
                        effective_dt,
                        pipe_speed,
                        bird.x,
                        bird.y,
                        powerups.is_magnet_active(),
                    );
                    if coins_got > 0 {
                        coins_this_game += coins_got;
                        let coin_bonus = coins_got * 2 * score_multiplier;
                        score += coin_bonus;
                        audio.play_score();
                        particles.add_sparkle(bird.x + BIRD_WIDTH / 2.0, bird.y - 10.0);
                        popups.add(
                            format!("+{} COIN", coin_bonus),
                            bird.x + BIRD_WIDTH / 2.0,
                            bird.y - 20.0,
                            Color::from_rgba(255, 215, 0, 255),
                            15.0,
                        );

                        if coins_this_game >= 10 {
                            achievements.try_unlock(Achievement::Collector);
                        }

                        check_score_events(score, difficulty, &mut weather, &mut achievements, &mut score_flash, combo);
                    }

                    // Live best score update
                    if score > best_score {
                        best_score = score;
                        save_high_score(best_score);
                    }

                    ground.update(effective_dt, pipe_speed);
                    clouds.update(effective_dt);
                    weather.update(dt);
                    particles.update(raw_dt);

                    // Collision checking
                    if game_mode == GameMode::TwoPlayer {
                        if alive1 && check_collision(&bird, &pipes) {
                            alive1 = false;
                            bird.start_death();
                            particles.add_explosion(bird.x + BIRD_WIDTH / 2.0, bird.y + BIRD_HEIGHT / 2.0);
                            audio.play_hit();
                            popups.add("P1 DOWN!", bird.x + 10.0, bird.y - 20.0, Color::from_rgba(255, 100, 100, 255), 16.0);
                        }
                        if alive2 && check_collision(&bird2, &pipes) {
                            alive2 = false;
                            bird2.start_death();
                            particles.add_explosion(bird2.x + BIRD_WIDTH / 2.0, bird2.y + BIRD_HEIGHT / 2.0);
                            audio.play_hit();
                            popups.add("P2 DOWN!", bird2.x + 10.0, bird2.y - 20.0, Color::from_rgba(100, 180, 255, 255), 16.0);
                        }
                        if !alive1 && !alive2 {
                            state = GameState::Dying;
                            screen_shake = 14.0;
                            combo = 0;
                            if score1 > score2 {
                                duel_wins_1 += 1;
                            } else if score2 > score1 {
                                duel_wins_2 += 1;
                            }
                            if score > best_score {
                                best_score = score;
                                save_high_score(best_score);
                            }
                            stats.record_game(score, coins_this_game);
                            save_score_history(score, coins_this_game, difficulty, skin);
                        }
                    } else {
                        // Solo collision checking
                        if !powerups.is_invulnerable() && check_collision(&bird, &pipes) {
                            if powerups.pop_shield() {
                                achievements.record_shield_used();
                                audio.play_shield_break();
                                particles.add_powerup_burst(
                                    bird.x + BIRD_WIDTH / 2.0,
                                    bird.y + BIRD_HEIGHT / 2.0,
                                    Color::from_rgba(64, 224, 255, 255),
                                );
                                screen_shake = 7.0;
                                score_flash.trigger("SHIELD BROKEN!");
                            } else {
                                state = GameState::Dying;
                                bird.start_death();
                                screen_shake = 14.0;
                                combo = 0;
                                particles.add_explosion(
                                    bird.x + BIRD_WIDTH / 2.0,
                                    bird.y + BIRD_HEIGHT / 2.0,
                                );
                                audio.play_hit();

                                if score > best_score {
                                    best_score = score;
                                    save_high_score(best_score);
                                }
                                stats.record_game(score, coins_this_game);
                                save_score_history(score, coins_this_game, difficulty, skin);

                                if stats.total_coins >= 50 {
                                    achievements.try_unlock(Achievement::CoinKing);
                                }
                                if stats.games_played >= 50 {
                                    achievements.try_unlock(Achievement::Veteran);
                                }
                            }
                        }
                    }
                }
            }

            GameState::Dying => {
                bird.update_death(dt);
                if game_mode == GameMode::TwoPlayer {
                    bird2.update_death(dt);
                }
                weather.update(dt);
                particles.update(raw_dt);

                let done = if game_mode == GameMode::TwoPlayer {
                    bird.death_done() && bird2.death_done()
                } else {
                    bird.death_done()
                };

                if done {
                    state = GameState::GameOver;
                }
            }

            GameState::GameOver => {
                weather.update(dt);
                particles.update(raw_dt);
                achievements.update(raw_dt);
                let has_touch = touches().iter().any(|t| t.phase == TouchPhase::Started);
                if is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::R)
                    || is_mouse_button_pressed(MouseButton::Left)
                    || has_touch
                {
                    state = GameState::Playing;
                    score = 0;
                    score1 = 0;
                    score2 = 0;
                    alive1 = true;
                    alive2 = true;
                    combo = 0;
                    coins_this_game = 0;
                    bird = if game_mode == GameMode::TwoPlayer {
                        Bird::new_with_skin_at(75.0, skin)
                    } else {
                        Bird::new_with_skin_at(BIRD_X, skin)
                    };
                    skin2 = if skin == BirdSkin::Blue { BirdSkin::Red } else { BirdSkin::Blue };
                    bird2 = Bird::new_with_skin_at(105.0, skin2);
                    pipes = PipeManager::new(difficulty.pipe_gap(), difficulty.speed_mult());
                    let initial_gaps = pipes.reset();
                    coin_mgr.reset();
                    powerups.reset();
                    achievements.reset_per_game();
                    for gap in initial_gaps.iter().skip(1) {
                        coin_mgr.spawn_for_pipe(gap.0, gap.1, gap.2);
                        powerups.spawn_for_pipe(gap.0, gap.1, gap.2);
                    }
                    bird.flap();
                    if game_mode == GameMode::TwoPlayer {
                        bird2.flap();
                    }
                    audio.play_flap();
                }
            }
        }

        // ── Screen shake decay ──────────────────────────────────────────
        if screen_shake > 0.0 {
            screen_shake = (screen_shake - dt * 0.8).max(0.0);
        }

        // ── Score flash white decay ─────────────────────────────────────
        if score_flash_white > 0.0 {
            score_flash_white -= raw_dt;
        }

        let shake_x = if screen_shake > 0.0 {
            ::macroquad::rand::gen_range(-screen_shake, screen_shake)
        } else { 0.0 };
        let shake_y = if screen_shake > 0.0 {
            ::macroquad::rand::gen_range(-screen_shake, screen_shake)
        } else { 0.0 };

        // ── Camera setup (full window or letterboxed) ───────────────
        let sw = screen_width();
        let sh = screen_height();

        let (vp_x, vp_y, vp_w, vp_h) = match screen_fit {
            ScreenFit::Fit => {
                let scale = (sw / GAME_WIDTH).min(sh / GAME_HEIGHT);
                let vp_w = (GAME_WIDTH * scale).round() as i32;
                let vp_h = (GAME_HEIGHT * scale).round() as i32;
                let vp_x = ((sw - vp_w as f32) / 2.0).round() as i32;
                let vp_y = ((sh - vp_h as f32) / 2.0).round() as i32;
                (vp_x, vp_y, vp_w, vp_h)
            }
            ScreenFit::Fill => (0, 0, sw.round() as i32, sh.round() as i32),
        };

        clear_background(BLACK);

        let cam = Camera2D {
            zoom: vec2(2.0 / GAME_WIDTH, 2.0 / GAME_HEIGHT),
            target: vec2(GAME_WIDTH / 2.0 + shake_x, GAME_HEIGHT / 2.0 + shake_y),
            viewport: Some((vp_x, vp_y, vp_w, vp_h)),
            ..Default::default()
        };
        set_camera(&cam);

        // ── Render layers ───────────────────────────────────────────────
        bg.draw(score, game_time);
        clouds.draw();

        // Speed lines at high pipe speed
        let pipe_speed = pipes.get_speed(score);
        let speed_intensity = ((pipe_speed - 3.5) / 2.5).clamp(0.0, 1.0);
        if state == GameState::Playing {
            draw_speed_lines(game_time, speed_intensity);
        }

        pipes.draw(game_time);
        coin_mgr.draw(game_time);
        powerups.draw_items(game_time);
        ground.draw();
        weather.draw(game_time);
        particles.draw();

        // Draw bird in all states except pure GameOver
        if state != GameState::GameOver {
            bird.draw_with_invuln(powerups.is_invulnerable(), game_time);
            if game_mode == GameMode::TwoPlayer {
                bird2.draw_with_invuln(false, game_time);
            }
            if state == GameState::Playing {
                powerups.draw_bird_auras(bird.x, bird.y, game_time);
            }
        }

        // SlowMo border vignette
        if powerups.is_slowmo_active() {
            draw_slowmo_vignette();
        }

        // Double score border vignette (orange tint)
        if powerups.is_double_score_active() {
            let edge_col = Color::from_rgba(255, 69, 0, 40);
            draw_rectangle(0.0, 0.0, GAME_WIDTH, 6.0, edge_col);
            draw_rectangle(0.0, GAME_HEIGHT - 6.0, GAME_WIDTH, 6.0, edge_col);
            draw_rectangle(0.0, 0.0, 6.0, GAME_HEIGHT, edge_col);
            draw_rectangle(GAME_WIDTH - 6.0, 0.0, 6.0, GAME_HEIGHT, edge_col);
        }

        // Floating score & bonus popups
        popups.draw();

        // ── HUD / Overlays ──────────────────────────────────────────────
        match state {
            GameState::Start => draw_start_screen(
                game_time,
                game_mode,
                difficulty,
                skin,
                weather.current,
                bg.mode,
                screen_fit,
                window_scale,
                is_fullscreen,
                audio.muted,
                &stats,
                &achievements,
            ),
            GameState::Playing => {
                let fps = if show_fps { Some(get_fps()) } else { None };
                if game_mode == GameMode::TwoPlayer {
                    draw_hud_2p(score1, score2, alive1, alive2, skin, skin2, coins_this_game, audio.muted, duel_wins_1, duel_wins_2);
                } else {
                    draw_hud(score, coins_this_game, fps, combo, audio.muted, &powerups);
                }
                score_flash.draw();
                achievements.draw_notification();

                // White flash on score
                if score_flash_white > 0.0 {
                    let a = (score_flash_white / 0.15).clamp(0.0, 1.0) * 0.25;
                    draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, Color::new(1.0, 1.0, 1.0, a));
                }

                if paused {
                    draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, Color::from_rgba(0, 0, 0, 100));
                    let p_text = "PAUSED";
                    let dim = measure_text(p_text, None, 36, 1.0);
                    draw_text(p_text, (GAME_WIDTH - dim.width) / 2.0, GAME_HEIGHT / 2.0, 36.0, WHITE);
                    let hint = "Press P to resume";
                    let h_dim = measure_text(hint, None, 16, 1.0);
                    draw_text(hint, (GAME_WIDTH - h_dim.width) / 2.0, GAME_HEIGHT / 2.0 + 30.0, 16.0,
                        Color::from_rgba(200, 200, 200, 200));
                }
            }
            GameState::Dying => {
                let fps = if show_fps { Some(get_fps()) } else { None };
                if game_mode == GameMode::TwoPlayer {
                    draw_hud_2p(score1, score2, alive1, alive2, skin, skin2, coins_this_game, audio.muted, duel_wins_1, duel_wins_2);
                } else {
                    draw_hud(score, coins_this_game, fps, 0, audio.muted, &powerups);
                }
                achievements.draw_notification();
            }
            GameState::GameOver => {
                if game_mode == GameMode::TwoPlayer {
                    draw_game_over_2p(score1, score2, skin, skin2, coins_this_game, duel_wins_1, duel_wins_2);
                } else {
                    draw_game_over(score, best_score, coins_this_game);
                    achievements.draw_summary();
                }
                achievements.draw_notification();
            }
        }

        set_default_camera();
        next_frame().await;
    }
}
