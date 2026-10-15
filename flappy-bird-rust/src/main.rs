mod constants;
mod bird;
mod pipes;
mod ground;
mod clouds;
mod particles;
mod background;
mod audio;
mod ui;

use macroquad::prelude::*;
use constants::*;
use bird::Bird;
use pipes::PipeManager;
use ground::Ground;
use clouds::CloudManager;
use particles::ParticleSystem;
use background::Background;
use audio::AudioManager;
use ui::{Difficulty, draw_start_screen, draw_game_over, draw_hud};

#[derive(PartialEq, Eq)]
enum GameState {
    Start,
    Playing,
    GameOver,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird - Rust Edition".to_string(),
        window_width: 400,
        window_height: 700,
        window_resizable: true,
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

fn check_collision(bird: &Bird, pipes: &PipeManager) -> bool {
    let margin = 3.0;
    let bx = bird.x + margin;
    let by = bird.y + margin;
    let bw = BIRD_WIDTH - margin * 2.0;
    let bh = BIRD_HEIGHT - margin * 2.0;

    // Ceiling & Ground
    if by <= 0.0 || by + bh >= GAME_HEIGHT - GROUND_HEIGHT {
        return true;
    }

    // Pipes
    for p in &pipes.pipes {
        if bx + bw > p.x && bx < p.x + p.width {
            if by < p.top_height || by + bh > p.bottom_y {
                return true;
            }
        }
    }

    false
}

#[macroquad::main(window_conf)]
async fn main() {
    let audio = AudioManager::new().await;
    let bg = Background::new();
    let mut clouds = CloudManager::new();
    let mut ground = Ground::new();
    let mut particles = ParticleSystem::new();
    let mut bird = Bird::new();
    
    let mut difficulty = Difficulty::Normal;
    let mut pipes = PipeManager::new(difficulty.pipe_gap(), difficulty.speed_mult());

    let mut state = GameState::Start;
    let mut score = 0u32;
    let mut best_score = load_high_score();
    let mut paused = false;
    let mut show_fps = false;

    let mut screen_shake = 0.0f32;
    let mut game_time = 0.0f32;

    loop {
        let raw_dt = get_frame_time();
        let dt = (raw_dt / 0.01667).min(3.0);
        game_time += raw_dt;

        // Toggle FPS
        if is_key_pressed(KeyCode::F) {
            show_fps = !show_fps;
        }

        // Toggle Pause
        if is_key_pressed(KeyCode::P) && state == GameState::Playing {
            paused = !paused;
        }

        // Difficulty change on start screen
        if state == GameState::Start && is_key_pressed(KeyCode::D) {
            difficulty = difficulty.next();
            pipes = PipeManager::new(difficulty.pipe_gap(), difficulty.speed_mult());
        }

        let tap = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Up)
            || is_mouse_button_pressed(MouseButton::Left);

        match state {
            GameState::Start => {
                clouds.update(dt);
                if tap {
                    state = GameState::Playing;
                    score = 0;
                    bird = Bird::new();
                    pipes.reset();
                    bird.flap();
                    audio.play_flap();
                }
            }
            GameState::Playing => {
                if !paused {
                    bird.update(dt);
                    particles.add_trail(bird.x + 5.0, bird.y + BIRD_HEIGHT / 2.0);

                    let scored = pipes.update(dt, &mut score, bird.x);
                    if scored {
                        audio.play_score();
                        particles.add_sparkle(bird.x + BIRD_WIDTH + 10.0, bird.y);
                    }

                    ground.update(dt, pipes.get_speed(score));
                    clouds.update(dt);
                    particles.update(raw_dt);

                    if tap {
                        bird.flap();
                        audio.play_flap();
                    }

                    if check_collision(&bird, &pipes) {
                        state = GameState::GameOver;
                        screen_shake = 12.0;
                        particles.add_explosion(bird.x + BIRD_WIDTH / 2.0, bird.y + BIRD_HEIGHT / 2.0);
                        audio.play_hit();

                        if score > best_score {
                            best_score = score;
                            save_high_score(best_score);
                        }
                    }
                }
            }
            GameState::GameOver => {
                particles.update(raw_dt);
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::R) || is_mouse_button_pressed(MouseButton::Left) {
                    state = GameState::Playing;
                    score = 0;
                    bird = Bird::new();
                    pipes.reset();
                    bird.flap();
                    audio.play_flap();
                }
            }
        }

        // Apply screen shake
        if screen_shake > 0.0 {
            screen_shake -= dt * 0.8;
            if screen_shake < 0.0 {
                screen_shake = 0.0;
            }
        }

        let shake_offset_x = if screen_shake > 0.0 {
            ::macroquad::rand::gen_range(-screen_shake, screen_shake)
        } else {
            0.0
        };
        let shake_offset_y = if screen_shake > 0.0 {
            ::macroquad::rand::gen_range(-screen_shake, screen_shake)
        } else {
            0.0
        };

        // Aspect ratio projection camera setup
        let screen_w = screen_width();
        let screen_h = screen_height();
        let scale = (screen_w / GAME_WIDTH).min(screen_h / GAME_HEIGHT);
        let vp_w = (GAME_WIDTH * scale) as i32;
        let vp_h = (GAME_HEIGHT * scale) as i32;
        let vp_x = ((screen_w - vp_w as f32) / 2.0) as i32;
        let vp_y = ((screen_h - vp_h as f32) / 2.0) as i32;

        clear_background(BLACK);

        let cam = Camera2D {
            zoom: vec2(2.0 / GAME_WIDTH, -2.0 / GAME_HEIGHT),
            target: vec2(GAME_WIDTH / 2.0 + shake_offset_x, GAME_HEIGHT / 2.0 + shake_offset_y),
            viewport: Some((vp_x, vp_y, vp_w, vp_h)),
            ..Default::default()
        };

        set_camera(&cam);

        // Rendering layers
        bg.draw(score, game_time);
        clouds.draw();
        pipes.draw(game_time);
        ground.draw();
        particles.draw();

        if state != GameState::GameOver {
            bird.draw();
        }

        match state {
            GameState::Start => draw_start_screen(game_time, difficulty),
            GameState::Playing => {
                let fps = if show_fps { Some(get_fps()) } else { None };
                draw_hud(score, fps);
                if paused {
                    draw_rectangle(0.0, 0.0, GAME_WIDTH, GAME_HEIGHT, Color::from_rgba(0, 0, 0, 100));
                    let p_text = "PAUSED";
                    let dim = measure_text(p_text, None, 36, 1.0);
                    draw_text(p_text, (GAME_WIDTH - dim.width) / 2.0, GAME_HEIGHT / 2.0, 36.0, WHITE);
                }
            }
            GameState::GameOver => {
                draw_game_over(score, best_score);
            }
        }

        set_default_camera();
        next_frame().await;
    }
}
