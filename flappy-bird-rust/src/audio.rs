use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

pub struct AudioManager {
    pub flap_sound: Option<Sound>,
    pub score_sound: Option<Sound>,
    pub hit_sound: Option<Sound>,
    pub powerup_sound: Option<Sound>,
    pub shield_break_sound: Option<Sound>,
    pub muted: bool,
}

fn create_wav(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let mut header = Vec::with_capacity(44 + samples.len() * 2);
    header.extend_from_slice(b"RIFF");
    let file_size = (36 + samples.len() * 2) as u32;
    header.extend_from_slice(&file_size.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes()); // PCM header length
    header.extend_from_slice(&1u16.to_le_bytes());  // Audio format 1 = PCM
    header.extend_from_slice(&1u16.to_le_bytes());  // 1 channel
    header.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2;
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&2u16.to_le_bytes());  // block align
    header.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    header.extend_from_slice(b"data");
    let data_len = (samples.len() * 2) as u32;
    header.extend_from_slice(&data_len.to_le_bytes());
    for &sample in samples {
        header.extend_from_slice(&sample.to_le_bytes());
    }
    header
}

impl AudioManager {
    pub async fn new() -> Self {
        let sr = 44100;

        // Flap tone: short rising sine wave
        let flap_len = (sr as f32 * 0.1) as usize;
        let mut flap_samples = Vec::with_capacity(flap_len);
        for i in 0..flap_len {
            let t = i as f32 / sr as f32;
            let freq = 400.0 + t * 2000.0;
            let env = 1.0 - (i as f32 / flap_len as f32);
            let val = ((t * freq * std::f32::consts::TAU).sin() * env * 14000.0) as i16;
            flap_samples.push(val);
        }
        let flap_bytes = create_wav(sr, &flap_samples);
        let flap_sound = load_sound_from_bytes(&flap_bytes).await.ok();

        // Score tone: chime
        let score_len = (sr as f32 * 0.18) as usize;
        let mut score_samples = Vec::with_capacity(score_len);
        for i in 0..score_len {
            let t = i as f32 / sr as f32;
            let freq = if t < 0.08 { 650.0 } else { 920.0 };
            let env = 1.0 - (i as f32 / score_len as f32);
            let val = ((t * freq * std::f32::consts::TAU).sin() * env * 16000.0) as i16;
            score_samples.push(val);
        }
        let score_bytes = create_wav(sr, &score_samples);
        let score_sound = load_sound_from_bytes(&score_bytes).await.ok();

        // Hit tone: descending low noise/buzz
        let hit_len = (sr as f32 * 0.25) as usize;
        let mut hit_samples = Vec::with_capacity(hit_len);
        for i in 0..hit_len {
            let t = i as f32 / sr as f32;
            let freq = (220.0 - t * 450.0).max(50.0);
            let env = 1.0 - (i as f32 / hit_len as f32);
            let val = if (t * freq * std::f32::consts::TAU).sin() > 0.0 {
                (env * 16000.0) as i16
            } else {
                (-env * 16000.0) as i16
            };
            hit_samples.push(val);
        }
        let hit_bytes = create_wav(sr, &hit_samples);
        let hit_sound = load_sound_from_bytes(&hit_bytes).await.ok();

        // Power-up pickup tone: rising arpeggio
        let pup_len = (sr as f32 * 0.22) as usize;
        let mut pup_samples = Vec::with_capacity(pup_len);
        for i in 0..pup_len {
            let t = i as f32 / sr as f32;
            let step = (t * 20.0) as i32;
            let freq = match step % 3 {
                0 => 523.25, // C5
                1 => 659.25, // E5
                _ => 783.99, // G5
            };
            let env = (1.0 - (i as f32 / pup_len as f32)).powf(0.5);
            let val = ((t * freq * std::f32::consts::TAU).sin() * env * 15000.0) as i16;
            pup_samples.push(val);
        }
        let pup_bytes = create_wav(sr, &pup_samples);
        let powerup_sound = load_sound_from_bytes(&pup_bytes).await.ok();

        // Shield break sound: sharp glass break / crunch
        let sb_len = (sr as f32 * 0.2) as usize;
        let mut sb_samples = Vec::with_capacity(sb_len);
        for i in 0..sb_len {
            let t = i as f32 / sr as f32;
            let freq = (800.0 - t * 1800.0).max(100.0);
            let env = (1.0 - (i as f32 / sb_len as f32)).powf(1.5);
            let noise = ((t * 12345.67).sin() * 5000.0) as i16;
            let tone = ((t * freq * std::f32::consts::TAU).sin() * env * 14000.0) as i16;
            sb_samples.push(tone.saturating_add(noise));
        }
        let sb_bytes = create_wav(sr, &sb_samples);
        let shield_break_sound = load_sound_from_bytes(&sb_bytes).await.ok();

        Self {
            flap_sound,
            score_sound,
            hit_sound,
            powerup_sound,
            shield_break_sound,
            muted: false,
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }

    pub fn play_flap(&self) {
        if self.muted {
            return;
        }
        if let Some(snd) = &self.flap_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_score(&self) {
        if self.muted {
            return;
        }
        if let Some(snd) = &self.score_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_hit(&self) {
        if self.muted {
            return;
        }
        if let Some(snd) = &self.hit_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_powerup(&self) {
        if self.muted {
            return;
        }
        if let Some(snd) = &self.powerup_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_shield_break(&self) {
        if self.muted {
            return;
        }
        if let Some(snd) = &self.shield_break_sound {
            play_sound_once(snd);
        }
    }
}
