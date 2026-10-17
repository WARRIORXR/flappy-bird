use macroquad::prelude::*;
use crate::constants::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimeOfDay {
    Auto,   // Cycles dynamically with score
    Day,
    Sunset,
    Night,
    Dawn,
}

impl TimeOfDay {
    pub fn name(&self) -> &'static str {
        match self {
            TimeOfDay::Auto => "AUTO",
            TimeOfDay::Day => "DAY",
            TimeOfDay::Sunset => "SUNSET",
            TimeOfDay::Night => "NIGHT",
            TimeOfDay::Dawn => "DAWN",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TimeOfDay::Auto => TimeOfDay::Day,
            TimeOfDay::Day => TimeOfDay::Sunset,
            TimeOfDay::Sunset => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::Dawn,
            TimeOfDay::Dawn => TimeOfDay::Auto,
        }
    }
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub brightness: f32,
    pub twinkle_speed: f32,
}

pub struct ShootingStar {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub length: f32,
    pub life: f32,
    pub max_life: f32,
}

pub struct Background {
    pub mode: TimeOfDay,
    pub stars: Vec<Star>,
    pub shooting_stars: Vec<ShootingStar>,
    pub shoot_timer: f32,
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

impl Background {
    pub fn new() -> Self {
        let mut stars = Vec::new();
        for _ in 0..50 {
            stars.push(Star {
                x: ::macroquad::rand::gen_range(0.0, GAME_WIDTH),
                y: ::macroquad::rand::gen_range(0.0, GAME_HEIGHT * 0.65),
                size: ::macroquad::rand::gen_range(1.0, 2.8),
                brightness: ::macroquad::rand::gen_range(0.4, 1.0),
                twinkle_speed: ::macroquad::rand::gen_range(2.0, 5.0),
            });
        }
        Self {
            mode: TimeOfDay::Auto,
            stars,
            shooting_stars: Vec::new(),
            shoot_timer: ::macroquad::rand::gen_range(3.0, 7.0),
        }
    }

    pub fn cycle_mode(&mut self) -> TimeOfDay {
        self.mode = self.mode.next();
        self.mode
    }

    pub fn update(&mut self, dt: f32) {
        // Update shooting stars
        for ss in &mut self.shooting_stars {
            ss.x += ss.vx * dt * 60.0;
            ss.y += ss.vy * dt * 60.0;
            ss.life -= dt;
        }
        self.shooting_stars.retain(|ss| ss.life > 0.0);

        self.shoot_timer -= dt;
        if self.shoot_timer <= 0.0 {
            self.shoot_timer = ::macroquad::rand::gen_range(4.0, 10.0);
            self.spawn_shooting_star();
        }
    }

    pub fn spawn_shooting_star(&mut self) {
        let start_x: f32 = ::macroquad::rand::gen_range(GAME_WIDTH * 0.2, GAME_WIDTH + 40.0);
        let start_y: f32 = ::macroquad::rand::gen_range(20.0, GAME_HEIGHT * 0.35);
        let speed: f32 = ::macroquad::rand::gen_range(5.0, 9.0);
        let angle: f32 = ::macroquad::rand::gen_range(2.4, 2.8); // down-left diagonal
        self.shooting_stars.push(ShootingStar {
            x: start_x,
            y: start_y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            length: ::macroquad::rand::gen_range(25.0, 55.0),
            life: 0.6,
            max_life: 0.6,
        });
    }

    pub fn draw(&self, score: u32, time: f32) {
        // Calculate continuous cycle factor 0.0 to 1.0 (repeats every 50 points)
        let cycle = match self.mode {
            TimeOfDay::Auto => ((score % 50) as f32 / 50.0).clamp(0.0, 1.0),
            TimeOfDay::Day => 0.35,
            TimeOfDay::Sunset => 0.65,
            TimeOfDay::Night => 0.90,
            TimeOfDay::Dawn => 0.08,
        };

        // 4 Key Sky Palettes: Dawn (0.0), Day (0.35), Sunset (0.65), Night (0.90)
        let (top_color, bot_color, night_alpha, sun_alpha, moon_alpha) = if cycle < 0.25 {
            // Dawn -> Day
            let t = cycle / 0.25;
            let dawn_top = Color::from_rgba(110, 80, 140, 255);
            let dawn_bot = Color::from_rgba(255, 170, 130, 255);
            let day_top = Color::from_rgba(125, 200, 245, 255);
            let day_bot = Color::from_rgba(215, 238, 255, 255);
            (
                lerp_color(dawn_top, day_top, t),
                lerp_color(dawn_bot, day_bot, t),
                (1.0 - t) * 0.5,
                t.clamp(0.0, 1.0),
                0.0,
            )
        } else if cycle < 0.55 {
            // Day
            let t = (cycle - 0.25) / 0.30;
            let day_top = Color::from_rgba(125, 200, 245, 255);
            let day_bot = Color::from_rgba(215, 238, 255, 255);
            let pre_sunset_top = Color::from_rgba(180, 150, 210, 255);
            let pre_sunset_bot = Color::from_rgba(255, 200, 140, 255);
            (
                lerp_color(day_top, pre_sunset_top, t),
                lerp_color(day_bot, pre_sunset_bot, t),
                0.0,
                1.0 - t * 0.3,
                0.0,
            )
        } else if cycle < 0.78 {
            // Sunset -> Twilight
            let t = (cycle - 0.55) / 0.23;
            let sun_top = Color::from_rgba(220, 90, 70, 255);
            let sun_bot = Color::from_rgba(255, 185, 95, 255);
            let twi_top = Color::from_rgba(40, 35, 80, 255);
            let twi_bot = Color::from_rgba(130, 65, 95, 255);
            (
                lerp_color(sun_top, twi_top, t),
                lerp_color(sun_bot, twi_bot, t),
                t * 0.8,
                (1.0 - t).max(0.0),
                t * 0.6,
            )
        } else {
            // Night
            let t = (cycle - 0.78) / 0.22;
            let twi_top = Color::from_rgba(40, 35, 80, 255);
            let twi_bot = Color::from_rgba(130, 65, 95, 255);
            let night_top = Color::from_rgba(14, 18, 48, 255);
            let night_bot = Color::from_rgba(30, 38, 75, 255);
            (
                lerp_color(twi_top, night_top, t),
                lerp_color(twi_bot, night_bot, t),
                0.8 + t * 0.2,
                0.0,
                1.0,
            )
        };

        // Draw sky gradient in vertical bands
        let bands = 16;
        let band_h = GAME_HEIGHT / (bands as f32);
        for i in 0..bands {
            let t = (i as f32) / (bands as f32);
            let col = Color::new(
                top_color.r * (1.0 - t) + bot_color.r * t,
                top_color.g * (1.0 - t) + bot_color.g * t,
                top_color.b * (1.0 - t) + bot_color.b * t,
                1.0,
            );
            draw_rectangle(0.0, (i as f32) * band_h, GAME_WIDTH, band_h + 1.0, col);
        }

        // Draw Sun during Day / Dawn / Sunset
        if sun_alpha > 0.05 {
            let sun_x = GAME_WIDTH * 0.78;
            let sun_y = 110.0 + (time * 0.4).sin() * 8.0;
            let pulse = 1.0 + (time * 2.5).sin() * 0.08;

            // Outer coronal glow
            draw_circle(sun_x, sun_y, 44.0 * pulse, Color::new(1.0, 0.85, 0.3, 0.15 * sun_alpha));
            draw_circle(sun_x, sun_y, 30.0 * pulse, Color::new(1.0, 0.9, 0.4, 0.3 * sun_alpha));
            // Core sun disc
            draw_circle(sun_x, sun_y, 20.0, Color::new(1.0, 0.95, 0.7, 0.95 * sun_alpha));

            // Rotating sun rays
            for k in 0..8 {
                let ray_ang = (k as f32) * std::f32::consts::PI / 4.0 + time * 0.2;
                let r1 = 24.0;
                let r2 = 34.0 + (time * 3.0 + k as f32).sin() * 4.0;
                let p1 = vec2(sun_x + ray_ang.cos() * r1, sun_y + ray_ang.sin() * r1);
                let p2 = vec2(sun_x + ray_ang.cos() * r2, sun_y + ray_ang.sin() * r2);
                draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, Color::new(1.0, 0.9, 0.5, 0.4 * sun_alpha));
            }
        }

        // Draw Moon during Twilight / Night
        if moon_alpha > 0.05 {
            let moon_x = GAME_WIDTH * 0.82;
            let moon_y = 100.0;
            let moon_radius = 22.0;

            // Soft atmospheric lunar halo
            draw_circle(moon_x, moon_y, moon_radius * 2.2, Color::new(0.65, 0.8, 1.0, 0.12 * moon_alpha));
            draw_circle(moon_x, moon_y, moon_radius * 1.5, Color::new(0.8, 0.9, 1.0, 0.2 * moon_alpha));

            // Moon disc
            draw_circle(moon_x, moon_y, moon_radius, Color::new(0.95, 0.95, 0.88, 0.95 * moon_alpha));
            // Crater cut-out to create a stylish crescent moon
            let cutout_x = moon_x + 9.0;
            let cutout_y = moon_y - 4.0;
            let cutout_r = moon_radius * 0.88;
            draw_circle(cutout_x, cutout_y, cutout_r, Color::new(top_color.r, top_color.g, top_color.b, moon_alpha));
        }

        // Stars appear during sunset/night
        if night_alpha > 0.05 {
            for s in &self.stars {
                let twinkle = 0.65 + (time * s.twinkle_speed + s.x).sin() * 0.35;
                let alpha = night_alpha * s.brightness * twinkle;
                draw_circle(s.x, s.y, s.size, Color::new(1.0, 1.0, 1.0, alpha));
            }

            // Draw shooting stars
            for ss in &self.shooting_stars {
                let tail_len = ss.length * (ss.life / ss.max_life);
                let alpha = (ss.life / ss.max_life).clamp(0.0, 1.0) * night_alpha;
                let speed_len = (ss.vx * ss.vx + ss.vy * ss.vy).sqrt().max(0.1);
                let dir_x = ss.vx / speed_len;
                let dir_y = ss.vy / speed_len;
                let tail_x = ss.x - dir_x * tail_len;
                let tail_y = ss.y - dir_y * tail_len;

                // Gradient tail
                draw_line(tail_x, tail_y, ss.x, ss.y, 2.0, Color::new(0.8, 0.95, 1.0, alpha * 0.8));
                draw_circle(ss.x, ss.y, 2.5, Color::new(1.0, 1.0, 1.0, alpha));
            }
        }
    }
}
