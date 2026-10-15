use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

pub struct AudioManager {
    pub flap_sound: Option<Sound>,
    pub score_sound: Option<Sound>,
    pub hit_sound: Option<Sound>,
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
        let sr = 22050;

        // Flap tone: short rising sine wave
        let flap_len = (sr as f32 * 0.1) as usize;
        let mut flap_samples = Vec::with_capacity(flap_len);
        for i in 0..flap_len {
            let t = i as f32 / sr as f32;
            let freq = 400.0 + t * 2000.0;
            let env = 1.0 - (i as f32 / flap_len as f32);
            let val = ((t * freq * std::f32::consts::TAU).sin() * env * 12000.0) as i16;
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
            let val = ((t * freq * std::f32::consts::TAU).sin() * env * 14000.0) as i16;
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
                (env * 15000.0) as i16
            } else {
                (-env * 15000.0) as i16
            };
            hit_samples.push(val);
        }
        let hit_bytes = create_wav(sr, &hit_samples);
        let hit_sound = load_sound_from_bytes(&hit_bytes).await.ok();

        Self {
            flap_sound,
            score_sound,
            hit_sound,
        }
    }

    pub fn play_flap(&self) {
        if let Some(snd) = &self.flap_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_score(&self) {
        if let Some(snd) = &self.score_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_hit(&self) {
        if let Some(snd) = &self.hit_sound {
            play_sound_once(snd);
        }
    }
}
