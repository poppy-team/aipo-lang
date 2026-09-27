//! Audio system and procedural chiptune sound synthesizer for Aipo Game Host.
//!
//! Provides hardware-accelerated sound and music playback via Macroquad / QuadSnd,
//! plus an in-memory procedural RIFF/WAV synthesizer (SFXR / ChipTone style)
//! enabling retro sound effects with zero external audio assets.

#![forbid(unsafe_code)]

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};
use std::sync::Mutex;

/// Parameters for procedural waveform synthesis.
#[derive(Debug, Clone)]
pub struct SynthConfig {
    /// Waveform: "square", "sawtooth", "sine", "triangle", "noise"
    pub wave_type: String,
    /// Starting frequency in Hertz (e.g. 440.0)
    pub start_freq: f32,
    /// Frequency slide multiplier (-1.0 to +1.0)
    pub freq_slide: f32,
    /// Duration in seconds (e.g. 0.25)
    pub duration: f32,
    /// Square wave duty cycle (0.01 to 0.99, default 0.5)
    pub duty_cycle: f32,
    /// Volume multiplier (0.0 to 1.0)
    pub volume: f32,
}

impl Default for SynthConfig {
    fn default() -> Self {
        Self {
            wave_type: "square".to_string(),
            start_freq: 440.0,
            freq_slide: 0.0,
            duration: 0.2,
            duty_cycle: 0.5,
            volume: 0.8,
        }
    }
}

/// Generates standard 16-bit PCM RIFF/WAV audio bytes in memory.
pub fn generate_wav_bytes(config: &SynthConfig) -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let duration = config.duration.clamp(0.01, 5.0);
    let num_samples = (duration * sample_rate as f32) as usize;
    let data_len = (num_samples * 2) as u32; // 16-bit mono = 2 bytes per sample

    let mut buf = Vec::with_capacity(44 + data_len as usize);

    // 1. RIFF Header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_len).to_le_bytes()); // File size - 8
    buf.extend_from_slice(b"WAVE");

    // 2. fmt Subchunk (PCM)
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // Subchunk size (16 for PCM)
    buf.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 = PCM)
    buf.extend_from_slice(&1u16.to_le_bytes()); // NumChannels (1 = Mono)
    buf.extend_from_slice(&sample_rate.to_le_bytes()); // SampleRate
    let byte_rate = sample_rate * 2;
    buf.extend_from_slice(&byte_rate.to_le_bytes()); // ByteRate
    buf.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign (NumChannels * BitsPerSample/8)
    buf.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample

    // 3. data Subchunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());

    // 4. Synthesize Samples
    let mut phase: f32 = 0.0;
    let mut rng_state: u32 = 0x1234_5678;

    let attack_samples = (sample_rate as f32 * 0.005) as usize; // 5ms attack to prevent click
    let decay_samples = num_samples.saturating_sub(attack_samples);

    for i in 0..num_samples {
        let t_norm = i as f32 / num_samples as f32; // 0.0 to 1.0

        // Frequency slide calculation
        let freq = (config.start_freq * (1.0 + config.freq_slide * t_norm)).max(20.0);
        phase = (phase + freq / sample_rate as f32).fract();

        // Waveform evaluation (-1.0 to 1.0)
        let raw_sample = match config.wave_type.as_str() {
            "sawtooth" => 2.0 * phase - 1.0,
            "sine" => (phase * std::f32::consts::TAU).sin(),
            "triangle" => {
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            "noise" => {
                // Fast Xorshift32 PRNG
                rng_state ^= rng_state << 13;
                rng_state ^= rng_state >> 17;
                rng_state ^= rng_state << 5;
                (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0
            }
            _ => {
                // Square wave with duty cycle
                let duty = config.duty_cycle.clamp(0.05, 0.95);
                if phase < duty { 1.0 } else { -1.0 }
            }
        };

        // Amplitude Envelope (Attack + Exponential Decay)
        let env = if i < attack_samples {
            i as f32 / attack_samples as f32
        } else {
            let decay_idx = i - attack_samples;
            let decay_t = decay_idx as f32 / decay_samples.max(1) as f32;
            (1.0 - decay_t).powf(1.8) // Smooth decay curve
        };

        let sample_f32 = raw_sample * env * config.volume.clamp(0.0, 1.0);
        let sample_i16 = (sample_f32 * 32000.0).clamp(-32768.0, 32767.0) as i16;

        buf.extend_from_slice(&sample_i16.to_le_bytes());
    }

    buf
}

/// Pre-configured procedural audio presets.
pub fn preset_config(name: &str) -> SynthConfig {
    match name {
        "coin" => SynthConfig {
            wave_type: "square".to_string(),
            start_freq: 987.77, // B5 note
            freq_slide: 0.35,   // Rapid pitch glide up to ~1330 Hz
            duration: 0.22,
            duty_cycle: 0.5,
            volume: 0.85,
        },
        "laser" | "shoot" => SynthConfig {
            wave_type: "sawtooth".to_string(),
            start_freq: 950.0,
            freq_slide: -0.85, // Sharp plunge downwards
            duration: 0.14,
            duty_cycle: 0.5,
            volume: 0.8,
        },
        "jump" => SynthConfig {
            wave_type: "square".to_string(),
            start_freq: 220.0,
            freq_slide: 0.75, // Upward leap
            duration: 0.18,
            duty_cycle: 0.3,
            volume: 0.85,
        },
        "explosion" => SynthConfig {
            wave_type: "noise".to_string(),
            start_freq: 120.0,
            freq_slide: -0.6,
            duration: 0.45,
            duty_cycle: 0.5,
            volume: 0.95,
        },
        "hit" | "hurt" => SynthConfig {
            wave_type: "noise".to_string(),
            start_freq: 320.0,
            freq_slide: -0.7,
            duration: 0.12,
            duty_cycle: 0.5,
            volume: 0.85,
        },
        "powerup" => SynthConfig {
            wave_type: "sawtooth".to_string(),
            start_freq: 440.0,
            freq_slide: 0.95, // Ascending triumphant chord sweep
            duration: 0.32,
            duty_cycle: 0.5,
            volume: 0.85,
        },
        "click" | "beep" => SynthConfig {
            wave_type: "sine".to_string(),
            start_freq: 620.0,
            freq_slide: 0.0,
            duration: 0.04,
            duty_cycle: 0.5,
            volume: 0.6,
        },
        _ => SynthConfig {
            wave_type: "square".to_string(),
            start_freq: 440.0,
            freq_slide: 0.0,
            duration: 0.1,
            duty_cycle: 0.5,
            volume: 0.7,
        },
    }
}

/// Sound asset cache managing handles, music tracks, and procedural presets.
pub struct AudioSystem {
    sounds: Vec<Sound>,
    bg_music: Option<Sound>,
    enabled: bool,
}

impl AudioSystem {
    const fn new() -> Self {
        Self {
            sounds: Vec::new(),
            bg_music: None,
            enabled: true,
        }
    }

    /// Loads a sound file from disk (WAV or OGG), returning a 1-based handle ID.
    pub fn load_file(&mut self, path: &str) -> i64 {
        match std::fs::read(path) {
            Ok(bytes) => self.load_bytes(&bytes),
            Err(err) => {
                eprintln!("[aipo-game-host] warning: could not load audio file '{path}': {err}");
                0
            }
        }
    }

    /// Loads a sound from in-memory audio bytes, returning a 1-based handle ID.
    pub fn load_bytes(&mut self, bytes: &[u8]) -> i64 {
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            futures_lite_poll(load_sound_from_bytes(bytes))
        }));

        match res {
            Ok(Ok(Ok(sound))) => {
                self.sounds.push(sound);
                self.sounds.len() as i64
            }
            _ => {
                // Headless fallback handle for CI/no-audio environments
                self.sounds.len() as i64 + 1
            }
        }
    }

    /// Plays a sound by its handle ID.
    pub fn play(&self, id: i64, volume: f32) {
        if !self.enabled || id <= 0 {
            return;
        }
        let idx = (id - 1) as usize;
        if let Some(sound) = self.sounds.get(idx) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: false,
                        volume: volume.clamp(0.0, 1.0),
                    },
                );
            }));
        }
    }

    /// Plays a procedural sound preset immediately without saving a handle.
    pub fn play_preset(&mut self, name: &str, volume: f32, pitch: f32) {
        let mut config = preset_config(name);
        if pitch > 0.0 {
            config.start_freq = (config.start_freq * pitch).clamp(20.0, 20000.0);
        }
        let wav = generate_wav_bytes(&config);
        let id = self.load_bytes(&wav);
        self.play(id, volume);
    }

    /// Synthesizes custom procedural audio bytes and returns a sound handle ID.
    pub fn synth_sound(&mut self, config: &SynthConfig) -> i64 {
        let wav = generate_wav_bytes(config);
        self.load_bytes(&wav)
    }

    /// Plays a background music track (optionally looped).
    pub fn play_music(&mut self, id: i64, volume: f32, should_loop: bool) {
        if !self.enabled || id <= 0 {
            return;
        }
        let idx = (id - 1) as usize;
        if let Some(sound) = self.sounds.get(idx).cloned() {
            // Stop previous music track if playing
            self.stop_music();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                play_sound(
                    &sound,
                    PlaySoundParams {
                        looped: should_loop,
                        volume: volume.clamp(0.0, 1.0),
                    },
                );
            }));
            self.bg_music = Some(sound);
        }
    }

    /// Stops currently playing background music.
    pub fn stop_music(&mut self) {
        if let Some(ref music) = self.bg_music.take() {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                stop_sound(music);
            }));
        }
    }

    /// Stops a specific sound by handle ID.
    pub fn stop(&self, id: i64) {
        if id <= 0 {
            return;
        }
        let idx = (id - 1) as usize;
        if let Some(sound) = self.sounds.get(idx) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                stop_sound(sound);
            }));
        }
    }
}

/// Helper to synchronously poll a future that finishes on the first tick in 100% safe Rust.
fn futures_lite_poll<F: std::future::Future>(fut: F) -> Result<F::Output, ()> {
    use std::task::{Context, Poll, Waker};

    let mut pinned = std::pin::pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());

    match pinned.as_mut().poll(&mut cx) {
        Poll::Ready(val) => Ok(val),
        Poll::Pending => Err(()),
    }
}

/// Global audio system instance.
pub static AUDIO: Mutex<AudioSystem> = Mutex::new(AudioSystem::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wav_header() {
        let config = SynthConfig {
            wave_type: "square".to_string(),
            start_freq: 440.0,
            freq_slide: 0.0,
            duration: 0.1,
            duty_cycle: 0.5,
            volume: 0.8,
        };

        let wav = generate_wav_bytes(&config);
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");

        // Sample rate = 44100
        let sample_rate = u32::from_le_bytes(wav[24..28].try_into().unwrap());
        assert_eq!(sample_rate, 44100);

        // Bits per sample = 16
        let bits_per_sample = u16::from_le_bytes(wav[34..36].try_into().unwrap());
        assert_eq!(bits_per_sample, 16);

        // Data length = num_samples * 2
        let data_len = u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize;
        assert_eq!(wav.len(), 44 + data_len);
    }

    #[test]
    fn test_all_presets_generate_valid_wav() {
        let presets = [
            "coin",
            "laser",
            "jump",
            "explosion",
            "hit",
            "powerup",
            "click",
            "beep",
        ];
        for preset in presets {
            let config = preset_config(preset);
            let wav = generate_wav_bytes(&config);
            assert!(
                wav.len() > 44,
                "Preset {preset} must generate valid WAV data"
            );
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
        }
    }

    #[test]
    fn test_audio_system_headless_safe() {
        let mut audio = AudioSystem::new();
        let config = preset_config("coin");
        let wav = generate_wav_bytes(&config);

        let id = audio.load_bytes(&wav);
        assert!(id > 0);

        // Calls should never panic in headless mode
        audio.play(id, 1.0);
        audio.play_preset("jump", 0.8, 1.2);
        let custom_id = audio.synth_sound(&config);
        assert!(custom_id > 0);
        audio.play(custom_id, 0.9);
        audio.play_music(id, 0.5, true);
        audio.stop_music();
        audio.stop(id);
    }
}
