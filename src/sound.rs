use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

const SAMPLE_RATE: u32 = 22_050;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SoundEvent {
    Start,
    Laser,
    Hit,
    Explosion,
    Damage,
    Dock,
    Error,
    Jump,
    Trade,
    Undock,
    Nav,
    GameOver,
}

const EVENTS: [SoundEvent; 12] = [
    SoundEvent::Start,
    SoundEvent::Laser,
    SoundEvent::Hit,
    SoundEvent::Explosion,
    SoundEvent::Damage,
    SoundEvent::Dock,
    SoundEvent::Error,
    SoundEvent::Jump,
    SoundEvent::Trade,
    SoundEvent::Undock,
    SoundEvent::Nav,
    SoundEvent::GameOver,
];

pub(crate) struct Audio {
    music: Option<Sound>,
    effects: Vec<(SoundEvent, Sound)>,
    pub(crate) muted: bool,
    playing: bool,
}

impl Audio {
    pub(crate) async fn new() -> Self {
        let music = load(&music_wav()).await;
        let mut effects = Vec::with_capacity(EVENTS.len());
        for event in EVENTS {
            if let Some(sound) = load(&effect_wav(event)).await {
                effects.push((event, sound));
            }
        }
        Self { music, effects, muted: false, playing: false }
    }

    pub(crate) fn toggle(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            if let Some(music) = &self.music { stop_sound(music); }
            for (_, effect) in &self.effects { stop_sound(effect); }
            self.playing = false;
        }
    }

    pub(crate) fn sync_music(&mut self, active: bool) {
        let should_play = active && !self.muted;
        if should_play == self.playing { return; }
        if let Some(music) = &self.music {
            if should_play {
                play_sound(music, PlaySoundParams { looped: true, volume: 0.24 });
            } else {
                stop_sound(music);
            }
        }
        self.playing = should_play;
    }

    pub(crate) fn play(&self, event: SoundEvent) {
        if self.muted { return; }
        if let Some((_, sound)) = self.effects.iter().find(|(kind, _)| *kind == event) {
            play_sound(sound, PlaySoundParams { looped: false, volume: 0.56 });
        }
    }
}

async fn load(bytes: &[u8]) -> Option<Sound> {
    match load_sound_from_bytes(bytes).await {
        Ok(sound) => Some(sound),
        Err(error) => {
            eprintln!("Could not load generated audio: {error}");
            None
        }
    }
}

fn note(midi: i32) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

fn pulse(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty { 1.0 - duty } else { -duty }
}

fn triangle(phase: f32) -> f32 {
    1.0 - 4.0 * (phase.fract() - 0.5).abs()
}

fn noise(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed as i32 as f32 / i32::MAX as f32
}

fn envelope(t: f32, duration: f32) -> f32 {
    (t / 0.004).min(1.0) * ((duration - t) / 0.025).clamp(0.0, 1.0)
}

fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_len as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }
    bytes
}

/// A sparse, original minor-key loop. All voices are generated, with no audio assets.
fn music_wav() -> Vec<u8> {
    const LEAD: [i32; 32] = [
        69, 0, 72, 76, 0, 79, 76, 0,
        67, 0, 71, 74, 0, 79, 74, 0,
        65, 0, 69, 72, 0, 76, 72, 0,
        67, 0, 71, 74, 0, 76, 74, 0,
    ];
    const ROOTS: [i32; 4] = [45, 43, 41, 43];
    let step_seconds = 60.0 / 112.0 / 2.0;
    let duration = LEAD.len() as f32 * step_seconds;
    let count = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut seed = 0x51a7_2026_u32;
    for i in 0..count {
        let t = i as f32 / SAMPLE_RATE as f32;
        let step = (t / step_seconds) as usize;
        let local = t - step as f32 * step_seconds;
        let gate = envelope(local, step_seconds * 0.85);
        let lead = if LEAD[step] == 0 { 0.0 } else {
            pulse(local * note(LEAD[step]), 0.25) * gate * 0.28
        };
        let root = ROOTS[step / 8];
        let bass = if step % 4 == 0 {
            triangle(local * note(root)) * envelope(local, step_seconds * 1.8) * 0.15
        } else { 0.0 };
        let arp_note = root + [12, 19, 24, 19][step % 4];
        let arp = pulse(local * note(arp_note), 0.5) * envelope(local, step_seconds * 0.42) * 0.05;
        let tick = if step % 2 == 0 { noise(&mut seed) * (-local * 75.0).exp() * 0.025 } else { 0.0 };
        samples.push((lead + bass + arp + tick) * envelope(t, duration));
    }
    wav(&samples)
}

fn effect_wav(event: SoundEvent) -> Vec<u8> {
    let duration = match event {
        SoundEvent::Start => 0.54,
        SoundEvent::Laser => 0.13,
        SoundEvent::Hit => 0.12,
        SoundEvent::Explosion => 0.62,
        SoundEvent::Damage => 0.35,
        SoundEvent::Dock => 0.58,
        SoundEvent::Error => 0.23,
        SoundEvent::Jump => 1.1,
        SoundEvent::Trade => 0.18,
        SoundEvent::Undock => 0.32,
        SoundEvent::Nav => 0.08,
        SoundEvent::GameOver => 1.2,
    };
    let count = (duration * SAMPLE_RATE as f32) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut phase = 0.0;
    let mut seed = 0xa31c_550f_u32;
    for i in 0..count {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = t / duration;
        let hz = match event {
            SoundEvent::Start => note([69, 72, 76, 81][((progress * 4.0) as usize).min(3)]),
            SoundEvent::Laser => 980.0 - 800.0 * progress,
            SoundEvent::Hit => 300.0 + 650.0 * progress,
            SoundEvent::Explosion => 110.0 - 65.0 * progress,
            SoundEvent::Damage => 310.0 - 220.0 * progress,
            SoundEvent::Dock => note([72, 76, 79, 84][((progress * 4.0) as usize).min(3)]),
            SoundEvent::Error => if progress < 0.5 { 175.0 } else { 130.0 },
            SoundEvent::Jump => 105.0 * 2.0_f32.powf(progress * 3.4),
            SoundEvent::Trade => note(if progress < 0.5 { 76 } else { 83 }),
            SoundEvent::Undock => 150.0 + 400.0 * progress,
            SoundEvent::Nav => 700.0,
            SoundEvent::GameOver => note([72, 69, 65, 60][((progress * 4.0) as usize).min(3)]),
        };
        phase += hz / SAMPLE_RATE as f32;
        let tone = if matches!(event, SoundEvent::Explosion | SoundEvent::Damage) {
            triangle(phase)
        } else {
            pulse(phase, if matches!(event, SoundEvent::Laser | SoundEvent::Jump) { 0.25 } else { 0.5 })
        };
        let grit = noise(&mut seed);
        let mixed = match event {
            SoundEvent::Explosion => tone * 0.25 + grit * 0.75,
            SoundEvent::Damage => tone * 0.65 + grit * 0.35,
            SoundEvent::Laser | SoundEvent::Jump => tone * 0.85 + grit * 0.15,
            _ => tone,
        };
        let decay = if matches!(event, SoundEvent::Explosion | SoundEvent::Damage | SoundEvent::Laser | SoundEvent::Nav) {
            (1.0 - progress).powf(1.4)
        } else { 0.8 };
        samples.push(mixed * envelope(t, duration) * decay * 0.62);
    }
    wav(&samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_clips_are_non_silent_pcm_without_clipping() {
        let clips = std::iter::once(music_wav()).chain(EVENTS.map(effect_wav));
        for bytes in clips {
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(&bytes[8..16], b"WAVEfmt ");
            assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize, bytes.len() - 44);
            let pcm: Vec<i16> = bytes[44..].chunks_exact(2)
                .map(|part| i16::from_le_bytes([part[0], part[1]])).collect();
            let peak = pcm.iter().map(|sample| (*sample as i32).abs()).max().unwrap();
            assert!(peak > 1000 && peak < 32767, "unexpected peak: {peak}");
            assert!(pcm.first().unwrap().abs() < 1500);
            assert!(pcm.last().unwrap().abs() < 1500);
        }
    }
}
