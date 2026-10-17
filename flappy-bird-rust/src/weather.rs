use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeatherType {
    Clear,
    Rain,
    Snow,
    Sakura,
    Firefly,
}

impl WeatherType {
    pub fn name(&self) -> &'static str {
        match self {
            WeatherType::Clear => "CLEAR",
            WeatherType::Rain => "RAIN",
            WeatherType::Snow => "SNOW",
            WeatherType::Sakura => "SAKURA",
            WeatherType::Firefly => "FIREFLY",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            WeatherType::Clear => WeatherType::Rain,
            WeatherType::Rain => WeatherType::Snow,
            WeatherType::Snow => WeatherType::Sakura,
            WeatherType::Sakura => WeatherType::Firefly,
            WeatherType::Firefly => WeatherType::Clear,
        }
    }
}

pub struct WeatherParticle {
    pub x: f32,
    pub y: f32,
    pub speed_y: f32,
    pub speed_x: f32,
    pub size: f32,
    pub rotation: f32,
    pub rot_speed: f32,
    pub phase: f32,
}

pub struct WeatherSystem {
    pub current: WeatherType,
    pub particles: Vec<WeatherParticle>,
}

impl WeatherSystem {
    pub fn new() -> Self {
        let mut sys = Self {
            current: WeatherType::Clear,
            particles: Vec::new(),
        };
        sys.reset_particles();
        sys
    }

    pub fn set_weather(&mut self, weather: WeatherType) {
        self.current = weather;
        self.reset_particles();
    }

    pub fn toggle_weather(&mut self) -> WeatherType {
        self.current = self.current.next();
        self.reset_particles();
        self.current
    }

    pub fn reset_particles(&mut self) {
        self.particles.clear();
        let count = match self.current {
            WeatherType::Clear => 0,
            WeatherType::Rain => 60,
            WeatherType::Snow => 50,
            WeatherType::Sakura => 35,
            WeatherType::Firefly => 25,
        };

        for _ in 0..count {
            self.particles.push(WeatherParticle {
                x: ::macroquad::rand::gen_range(0.0, GAME_WIDTH + 60.0),
                y: ::macroquad::rand::gen_range(0.0, GAME_HEIGHT),
                speed_y: match self.current {
                    WeatherType::Rain => ::macroquad::rand::gen_range(7.0, 13.0),
                    WeatherType::Snow => ::macroquad::rand::gen_range(1.0, 2.5),
                    WeatherType::Sakura => ::macroquad::rand::gen_range(1.2, 2.8),
                    WeatherType::Firefly => ::macroquad::rand::gen_range(-0.5, 0.5),
                    WeatherType::Clear => 0.0,
                },
                speed_x: match self.current {
                    WeatherType::Rain => -2.5,
                    WeatherType::Snow => ::macroquad::rand::gen_range(-0.5, 0.5),
                    WeatherType::Sakura => ::macroquad::rand::gen_range(-1.2, -0.4),
                    WeatherType::Firefly => ::macroquad::rand::gen_range(-0.8, 0.8),
                    WeatherType::Clear => 0.0,
                },
                size: match self.current {
                    WeatherType::Rain => ::macroquad::rand::gen_range(9.0, 16.0),
                    WeatherType::Snow => ::macroquad::rand::gen_range(1.5, 3.5),
                    WeatherType::Sakura => ::macroquad::rand::gen_range(3.0, 6.0),
                    WeatherType::Firefly => ::macroquad::rand::gen_range(2.0, 4.0),
                    WeatherType::Clear => 0.0,
                },
                rotation: ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU),
                rot_speed: ::macroquad::rand::gen_range(-2.0, 2.0),
                phase: ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU),
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.current == WeatherType::Clear {
            return;
        }

        for p in &mut self.particles {
            match self.current {
                WeatherType::Rain => {
                    p.x += p.speed_x * dt;
                    p.y += p.speed_y * dt;
                }
                WeatherType::Snow => {
                    p.phase += dt * 0.05;
                    p.x += (p.speed_x + p.phase.sin() * 0.8) * dt;
                    p.y += p.speed_y * dt;
                }
                WeatherType::Sakura => {
                    p.phase += dt * 0.04;
                    p.x += (p.speed_x + (p.phase * 1.5).sin() * 1.2) * dt;
                    p.y += p.speed_y * dt;
                    p.rotation += p.rot_speed * dt;
                }
                WeatherType::Firefly => {
                    p.phase += dt * 0.03;
                    p.x += (p.speed_x + (p.phase * 2.0).sin() * 1.5) * dt;
                    p.y += (p.speed_y + (p.phase * 1.3).cos() * 0.8) * dt;
                    // Wrap around screen
                    if p.y > GAME_HEIGHT - GROUND_HEIGHT {
                        p.y = 10.0;
                    }
                    if p.y < 0.0 {
                        p.y = GAME_HEIGHT - GROUND_HEIGHT - 20.0;
                    }
                    if p.x < -10.0 {
                        p.x = GAME_WIDTH + 5.0;
                    }
                    if p.x > GAME_WIDTH + 10.0 {
                        p.x = -5.0;
                    }
                }
                WeatherType::Clear => {}
            }

            // Wrap around bottom and edges (for non-firefly types)
            if self.current != WeatherType::Firefly {
                if p.y > GAME_HEIGHT - GROUND_HEIGHT + 10.0 {
                    p.y = -15.0;
                    p.x = ::macroquad::rand::gen_range(0.0, GAME_WIDTH + 60.0);
                }
                if p.x < -20.0 {
                    p.x = GAME_WIDTH + 10.0;
                }
            }
        }
    }

    pub fn draw(&self, time: f32) {
        match self.current {
            WeatherType::Clear => {}
            WeatherType::Rain => {
                let rain_color = Color::from_rgba(180, 215, 255, 170);
                for p in &self.particles {
                    draw_line(p.x, p.y, p.x + p.speed_x * 1.2, p.y + p.size, 1.3, rain_color);
                }
            }
            WeatherType::Snow => {
                let snow_color = Color::from_rgba(255, 255, 255, 200);
                for p in &self.particles {
                    draw_circle(p.x, p.y, p.size, snow_color);
                }
            }
            WeatherType::Sakura => {
                for p in &self.particles {
                    let petal_col = Color::from_rgba(255, 183, 197, 210);
                    draw_poly(p.x, p.y, 8, p.size, p.rotation.to_degrees(), petal_col);
                }
            }
            WeatherType::Firefly => {
                for p in &self.particles {
                    let glow = 0.5 + (time * 3.0 + p.phase).sin().abs() * 0.5;
                    // Outer glow
                    draw_circle(p.x, p.y, p.size * 2.5, Color::new(0.4, 1.0, 0.2, 0.15 * glow));
                    // Core
                    draw_circle(p.x, p.y, p.size, Color::new(0.6, 1.0, 0.3, glow));
                }
            }
        }
    }
}
