#[cfg(not(target_os = "windows"))]
use macroquad::audio::play_sound_once;
use macroquad::audio::{load_sound_from_bytes, Sound};

#[cfg(target_os = "windows")]
mod win_audio {
    #[link(name = "winmm")]
    extern "system" {
        fn PlaySoundA(psz_sound: *const u8, hmod: *mut std::ffi::c_void, fdw_sound: u32) -> i32;
    }
    const SND_ASYNC: u32 = 0x0001;
    const SND_NODEFAULT: u32 = 0x0002;
    const SND_MEMORY: u32 = 0x0004;

    pub fn play_wav(bytes: &'static [u8]) {
        unsafe {
            PlaySoundA(bytes.as_ptr(), std::ptr::null_mut(), SND_MEMORY | SND_ASYNC | SND_NODEFAULT);
        }
    }

    pub fn stop_sound() {
        unsafe {
            PlaySoundA(std::ptr::null(), std::ptr::null_mut(), 0);
        }
    }
}

pub struct AudioManager {
    #[allow(dead_code)]
    pub flap_sound: Option<Sound>,
    #[allow(dead_code)]
    pub score_sound: Option<Sound>,
    #[allow(dead_code)]
    pub hit_sound: Option<Sound>,
    #[allow(dead_code)]
    pub powerup_sound: Option<Sound>,
    #[allow(dead_code)]
    pub shield_break_sound: Option<Sound>,
    #[allow(dead_code)]
    pub golden_sound: Option<Sound>,
    #[allow(dead_code)]
    pub close_shave_sound: Option<Sound>,
    pub flap_wav: &'static [u8],
    pub score_wav: &'static [u8],
    pub hit_wav: &'static [u8],
    pub powerup_wav: &'static [u8],
    pub shield_break_wav: &'static [u8],
    pub golden_wav: &'static [u8],
    pub close_shave_wav: &'static [u8],
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

        // Golden pipe score fanfare: 4-note ascending major arpeggio
        let gold_len = (sr as f32 * 0.32) as usize;
        let mut gold_samples = Vec::with_capacity(gold_len);
        for i in 0..gold_len {
            let t = i as f32 / sr as f32;
            let step = (t * 16.0) as usize;
            let freq = match step.min(3) {
                0 => 523.25,  // C5
                1 => 659.25,  // E5
                2 => 783.99,  // G5
                _ => 1046.50, // C6
            };
            let env = (1.0 - (i as f32 / gold_len as f32)).powf(0.8);
            let tone = ((t * freq * std::f32::consts::TAU).sin() * env * 16000.0) as i16;
            gold_samples.push(tone);
        }
        let gold_bytes = create_wav(sr, &gold_samples);
        let golden_sound = load_sound_from_bytes(&gold_bytes).await.ok();

        // Close shave sound: fast whoosh chirp
        let cs_len = (sr as f32 * 0.12) as usize;
        let mut cs_samples = Vec::with_capacity(cs_len);
        for i in 0..cs_len {
            let t = i as f32 / sr as f32;
            let freq = 1100.0 + t * 2400.0;
            let env = (1.0 - (i as f32 / cs_len as f32)).powf(1.2);
            let val = ((t * freq * std::f32::consts::TAU).sin() * env * 13000.0) as i16;
            cs_samples.push(val);
        }
        let cs_bytes = create_wav(sr, &cs_samples);
        let close_shave_sound = load_sound_from_bytes(&cs_bytes).await.ok();

        let flap_wav = Box::leak(flap_bytes.into_boxed_slice());
        let score_wav = Box::leak(score_bytes.into_boxed_slice());
        let hit_wav = Box::leak(hit_bytes.into_boxed_slice());
        let pup_wav = Box::leak(pup_bytes.into_boxed_slice());
        let sb_wav = Box::leak(sb_bytes.into_boxed_slice());
        let golden_wav = Box::leak(gold_bytes.into_boxed_slice());
        let cs_wav = Box::leak(cs_bytes.into_boxed_slice());

        Self {
            flap_sound,
            score_sound,
            hit_sound,
            powerup_sound,
            shield_break_sound,
            golden_sound,
            close_shave_sound,
            flap_wav,
            score_wav,
            hit_wav,
            powerup_wav: pup_wav,
            shield_break_wav: sb_wav,
            golden_wav,
            close_shave_wav: cs_wav,
            muted: false,
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        if self.muted {
            #[cfg(target_os = "windows")]
            win_audio::stop_sound();
        }
        self.muted
    }

    pub fn play_flap(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.flap_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.flap_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_score(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.score_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.score_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_golden(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.golden_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.golden_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_close_shave(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.close_shave_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.close_shave_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_hit(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.hit_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.hit_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_powerup(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.powerup_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.powerup_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_shield_break(&self) {
        if self.muted {
            return;
        }
        #[cfg(target_os = "windows")]
        win_audio::play_wav(self.shield_break_wav);

        #[cfg(not(target_os = "windows"))]
        if let Some(snd) = &self.shield_break_sound {
            play_sound_once(snd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wav_validity() {
        let sr = 44100;
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
        println!("Generated WAV length: {} bytes", flap_bytes.len());
        println!("Header: {:?}", &flap_bytes[..44]);

        #[cfg(target_os = "windows")]
        {
            #[link(name = "winmm")]
            extern "system" {
                fn PlaySoundA(psz_sound: *const u8, hmod: *mut std::ffi::c_void, fdw_sound: u32) -> i32;
            }
            const SND_SYNC: u32 = 0x0000;
            const SND_MEMORY: u32 = 0x0004;
            let res = unsafe { PlaySoundA(flap_bytes.as_ptr(), std::ptr::null_mut(), SND_MEMORY | SND_SYNC) };
            println!("PlaySoundA result: {}", res);
            assert_ne!(res, 0, "PlaySoundA failed");
        }
    }
}



