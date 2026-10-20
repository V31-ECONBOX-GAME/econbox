//! Sounds synthesized in code: the button click, the tab slide and the puff of a summoned cloud.

use crate::options::Options;
use bevy::audio::{AddAudioSource, AudioPlugin, ChannelCount, SampleRate, Source, Volume};
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use std::time::Duration;

pub const RATE: u32 = 44_100;
const CLICK_LENGTH: f32 = 0.045;
const CLICK_PEAK: f32 = 0.35;
const SLIDE_LENGTH: f32 = 0.22;
const SLIDE_PEAK: f32 = 0.22;
const PUFF_LENGTH: f32 = 0.7;
const PUFF_PEAK: f32 = 0.3;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    Click,
    Slide,
    Cloud,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct Chime {
    samples: Arc<[f32]>,
}

impl Chime {
    pub fn new(samples: Vec<f32>) -> Self {
        Self {
            samples: samples.into(),
        }
    }
}

pub struct Playback {
    samples: Arc<[f32]>,
    at: usize,
}

impl Iterator for Playback {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.samples.get(self.at).copied();
        self.at += 1;
        sample
    }
}

impl Source for Playback {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::MIN
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(RATE).expect("a sample rate")
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.samples.len() as f32 / RATE as f32,
        ))
    }
}

impl Decodable for Chime {
    type Decoder = Playback;

    fn decoder(&self) -> Playback {
        Playback {
            samples: self.samples.clone(),
            at: 0,
        }
    }
}

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

fn level(mut wave: Vec<f32>, peak: f32) -> Vec<f32> {
    let loudest = wave
        .iter()
        .fold(0.0f32, |most, sample| most.max(sample.abs()));

    if loudest > 0.0 {
        for sample in &mut wave {
            *sample *= peak / loudest;
        }
    }

    wave
}

pub fn click() -> Vec<f32> {
    let length = (RATE as f32 * CLICK_LENGTH) as usize;
    let mut noise = Noise(0x2545_f491);
    let mut phase = 0.0f32;

    let wave = (0..length)
        .map(|index| {
            let t = index as f32 / RATE as f32;
            let pitch = 1500.0 + 900.0 * (-t / 0.004).exp();
            phase = (phase + TAU * pitch / RATE as f32) % TAU;
            let attack = (t / 0.001).min(1.0);
            let release = 1.0 - index as f32 / length as f32;
            let tone = phase.sin() * (-t / 0.009).exp();
            let tick = noise.next() * (-t / 0.0015).exp();
            attack * release * (0.7 * tone + 0.4 * tick)
        })
        .collect();

    level(wave, CLICK_PEAK)
}

pub fn slide() -> Vec<f32> {
    let length = (RATE as f32 * SLIDE_LENGTH) as usize;
    let mut noise = Noise(0x9e37_79b9);
    let mut low = 0.0f32;
    let mut phase = 0.0f32;

    let wave = (0..length)
        .map(|index| {
            let u = index as f32 / length as f32;
            let cutoff = 500.0 + 2500.0 * u;
            let pull = 1.0 - (-TAU * cutoff / RATE as f32).exp();
            low += (noise.next() - low) * pull;
            let pitch = 260.0 + 220.0 * u;
            phase = (phase + TAU * pitch / RATE as f32) % TAU;
            let swell = (PI * u.powf(0.6)).sin().powi(2);
            swell * (low + 0.15 * phase.sin())
        })
        .collect();

    level(wave, SLIDE_PEAK)
}

pub fn puff() -> Vec<f32> {
    let length = (RATE as f32 * PUFF_LENGTH) as usize;
    let mut noise = Noise(0x6c07_8965);
    let mut low = 0.0f32;
    let mut lower = 0.0f32;
    let mut phase = 0.0f32;

    let wave = (0..length)
        .map(|index| {
            let t = index as f32 / RATE as f32;
            let u = index as f32 / length as f32;
            let cutoff = 300.0 + 1500.0 * (-t / 0.12).exp();
            let pull = 1.0 - (-TAU * cutoff / RATE as f32).exp();
            low += (noise.next() - low) * pull;
            lower += (low - lower) * pull;
            let pitch = 70.0 + 50.0 * (-t / 0.15).exp();
            phase = (phase + TAU * pitch / RATE as f32) % TAU;
            let attack = (t / 0.04).min(1.0).powi(2);
            let release = (1.0 - u).powi(2);
            let body = (-t / 0.22).exp();
            attack * release * body * (lower + 0.25 * phase.sin())
        })
        .collect();

    level(wave, PUFF_PEAK)
}

#[derive(Resource, Debug, Clone)]
pub struct Sounds {
    click: Handle<Chime>,
    slide: Handle<Chime>,
    puff: Handle<Chime>,
}

pub fn register(app: &mut App) {
    app.add_message::<Cue>();

    if app.is_plugin_added::<AudioPlugin>() {
        app.add_audio_source::<Chime>().add_systems(Startup, forge);
    }
}

fn forge(mut commands: Commands, mut chimes: ResMut<Assets<Chime>>) {
    commands.insert_resource(Sounds {
        click: chimes.add(Chime::new(click())),
        slide: chimes.add(Chime::new(slide())),
        puff: chimes.add(Chime::new(puff())),
    });
}

pub fn loudness(options: &Options) -> f32 {
    options.percent("volume_master_sound") * options.percent("volume_ui")
}

pub fn effects(options: &Options) -> f32 {
    options.percent("volume_master_sound") * options.percent("volume_sound_effects")
}

pub fn play(
    mut commands: Commands,
    mut cues: MessageReader<Cue>,
    sounds: Option<Res<Sounds>>,
    options: Res<Options>,
) {
    let mut heard = Vec::new();

    for cue in cues.read() {
        if !heard.contains(cue) {
            heard.push(*cue);
        }
    }

    let Some(sounds) = sounds else { return };

    for cue in heard {
        let (chime, volume) = match cue {
            Cue::Click => (&sounds.click, loudness(&options)),
            Cue::Slide => (&sounds.slide, loudness(&options)),
            Cue::Cloud => (&sounds.puff, effects(&options)),
        };

        if volume <= 0.0 {
            continue;
        }

        commands.spawn((
            AudioPlayer(chime.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peak(wave: &[f32]) -> f32 {
        wave.iter().fold(0.0, |most, sample| most.max(sample.abs()))
    }

    #[test]
    fn the_click_is_short_and_the_slide_is_soft() {
        let click = click();
        let slide = slide();

        assert!(click.len() < slide.len());
        assert!((peak(&click) - CLICK_PEAK).abs() < 1e-4);
        assert!((peak(&slide) - SLIDE_PEAK).abs() < 1e-4);
        assert!(slide.iter().chain(&click).all(|sample| sample.is_finite()));
    }

    #[test]
    fn the_cloud_puff_is_the_longest_and_follows_the_effects_volume() {
        let puff = puff();
        let mut options = Options::default();

        assert!(puff.len() > slide().len());
        assert!((peak(&puff) - PUFF_PEAK).abs() < 1e-4);
        assert!(puff.iter().all(|sample| sample.is_finite()));

        options.set("volume_sound_effects", crate::options::Value::Int(0));
        assert_eq!(effects(&options), 0.0);
        assert!(loudness(&options) > 0.0);
    }

    #[test]
    fn every_sound_starts_and_ends_near_silence() {
        for wave in [click(), slide(), puff()] {
            assert!(wave[0].abs() < 0.01, "{}", wave[0]);
            assert!(
                wave[wave.len() - 1].abs() < 0.01,
                "{}",
                wave[wave.len() - 1]
            );
        }
    }

    #[test]
    fn a_chime_plays_its_samples_once() {
        let chime = Chime::new(vec![0.1, 0.2, 0.3]);
        let played: Vec<f32> = chime.decoder().collect();

        assert_eq!(played, vec![0.1, 0.2, 0.3]);
        assert_eq!(chime.decoder().channels(), ChannelCount::MIN);
    }
}
