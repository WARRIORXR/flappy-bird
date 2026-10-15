use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub color: Color,
    pub life: f32,
    pub max_life: f32,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    pub fn add_trail(&mut self, x: f32, y: f32) {
        self.particles.push(Particle {
            x,
            y,
            vx: ::macroquad::rand::gen_range(-1.0, -0.2),
            vy: ::macroquad::rand::gen_range(-0.5, 0.5),
            size: ::macroquad::rand::gen_range(2.0, 4.5),
            color: Color::from_rgba(255, 215, 0, 200),
            life: 0.35,
            max_life: 0.35,
        });
    }

    pub fn add_explosion(&mut self, x: f32, y: f32) {
        for _ in 0..30 {
            let angle = ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = ::macroquad::rand::gen_range(1.5, 5.5);
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                size: ::macroquad::rand::gen_range(3.0, 6.0),
                color: Color::from_rgba(255, 80, 20, 255),
                life: 0.6,
                max_life: 0.6,
            });
        }
    }

    pub fn add_sparkle(&mut self, x: f32, y: f32) {
        for _ in 0..15 {
            let angle = ::macroquad::rand::gen_range(0.0, std::f32::consts::TAU);
            let speed = ::macroquad::rand::gen_range(1.0, 3.5);
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                size: ::macroquad::rand::gen_range(2.0, 4.0),
                color: Color::from_rgba(255, 255, 100, 255),
                life: 0.45,
                max_life: 0.45,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut col = p.color;
            col.a *= alpha;
            draw_circle(p.x, p.y, p.size * alpha, col);
        }
    }
}
