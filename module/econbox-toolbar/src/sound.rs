//! Sounds synthesized in code: the button click, the tab slide, the puff of a summoned cloud and the drop landings.

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
pub const SPLASH_VOICES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Voice {
    Rain,
    Acid,
    Fire,
    Lava,
    Snow,
    Ash,
    Magic,
    Rage,
    Seed,
}

pub const VOICES: [Voice; 9] = [
    Voice::Rain,
    Voice::Acid,
    Voice::Fire,
    Voice::Lava,
    Voice::Snow,
    Voice::Ash,
    Voice::Magic,
    Voice::Rage,
    Voice::Seed,
];

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Splash {
    pub voice: Voice,
    pub volume: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sounding(pub Voice);

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

struct Tone {
    phase: f32,
}

impl Tone {
    fn at(&mut self, pitch: f32) -> f32 {
        self.phase = (self.phase + TAU * pitch / RATE as f32) % TAU;
        self.phase.sin()
    }
}

struct Smooth(f32);

impl Smooth {
    fn pass(&mut self, input: f32, cutoff: f32) -> f32 {
        let pull = 1.0 - (-TAU * cutoff / RATE as f32).exp();
        self.0 += (input - self.0) * pull;
        self.0
    }
}

fn shape(seconds: f32, peak: f32, mut sample: impl FnMut(f32, f32) -> f32) -> Vec<f32> {
    let length = (RATE as f32 * seconds) as usize;
    let wave = (0..length)
        .map(|index| {
            let t = index as f32 / RATE as f32;
            let u = index as f32 / length as f32;
            let attack = (t / 0.003).min(1.0);
            let release = (1.0 - u).powi(2);
            attack * release * sample(t, u)
        })
        .collect();
    level(wave, peak)
}

pub fn splash(voice: Voice) -> Vec<f32> {
    match voice {
        Voice::Rain => {
            let mut tone = Tone { phase: 0.0 };
            let mut noise = Noise(0x1b87_3593);
            shape(0.12, 0.16, |t, _| {
                let pitch = 900.0 + 1400.0 * (1.0 - (-t / 0.02).exp());
                tone.at(pitch) * (-t / 0.03).exp() + 0.2 * noise.next() * (-t / 0.004).exp()
            })
        }
        Voice::Acid => {
            let mut noise = Noise(0xcc9e_2d51);
            let mut low = Smooth(0.0);
            let mut tone = Tone { phase: 0.0 };
            shape(0.38, 0.14, |t, u| {
                let hiss = noise.next();
                let fizz = hiss - low.pass(hiss, 2500.0);
                let crackle = if (t * 90.0).fract() < 0.08 { 1.0 } else { 0.4 };
                fizz * crackle * (1.0 - u * 0.5)
                    + 0.15 * tone.at(520.0 - 200.0 * u) * (-t / 0.05).exp()
            })
        }
        Voice::Fire => {
            let mut noise = Noise(0xe654_6b64);
            let mut low = Smooth(0.0);
            let mut pops = Noise(0x85eb_ca6b);
            shape(0.32, 0.2, |t, _| {
                let rush = low.pass(noise.next(), 600.0 + 2200.0 * (-t / 0.06).exp());
                let pop = if pops.next() > 0.995 {
                    pops.next()
                } else {
                    0.0
                };
                rush * (-t / 0.12).exp() * 2.5 + pop
            })
        }
        Voice::Lava => {
            let mut tone = Tone { phase: 0.0 };
            let mut noise = Noise(0xc2b2_ae35);
            let mut low = Smooth(0.0);
            shape(0.42, 0.22, |t, _| {
                let pitch = 70.0 + 110.0 * (-t / 0.05).exp();
                let bubble = low.pass(noise.next(), 300.0);
                tone.at(pitch) * (-t / 0.18).exp() + bubble * 2.0 * (-t / 0.1).exp()
            })
        }
        Voice::Snow => {
            let mut noise = Noise(0x27d4_eb2f);
            let mut low = Smooth(0.0);
            let mut lower = Smooth(0.0);
            shape(0.2, 0.08, |t, _| {
                lower.pass(low.pass(noise.next(), 900.0), 900.0) * (-t / 0.06).exp()
            })
        }
        Voice::Ash => {
            let mut noise = Noise(0x1656_67b1);
            let mut low = Smooth(0.0);
            let mut band = Smooth(0.0);
            shape(0.22, 0.09, |t, _| {
                let grain = noise.next();
                let dust = low.pass(grain, 1800.0) - band.pass(grain, 400.0);
                dust * (-t / 0.07).exp()
            })
        }
        Voice::Magic => {
            let mut first = Tone { phase: 0.0 };
            let mut second = Tone { phase: 0.0 };
            let mut third = Tone { phase: 0.0 };
            shape(0.45, 0.14, |t, _| {
                let shimmer = 1.0 + 0.3 * (TAU * 14.0 * t).sin();
                (first.at(1320.0) + 0.6 * second.at(1980.0) + 0.35 * third.at(2640.0))
                    * shimmer
                    * (-t / 0.16).exp()
            })
        }
        Voice::Rage => {
            let mut tone = Tone { phase: 0.0 };
            let mut grit = Tone { phase: 0.0 };
            let mut noise = Noise(0x9e37_79b1);
            shape(0.3, 0.2, |t, _| {
                let pitch = 95.0 + 40.0 * (-t / 0.04).exp();
                let body =
                    (tone.at(pitch) * 3.0).tanh() + 0.5 * (grit.at(pitch * 1.5) * 4.0).tanh();
                (body + 0.3 * noise.next()) * (-t / 0.11).exp()
            })
        }
        Voice::Seed => {
            let mut tone = Tone { phase: 0.0 };
            let mut noise = Noise(0x7f4a_7c15);
            shape(0.2, 0.12, |t, u| {
                let pitch = 560.0 + 420.0 * u;
                tone.at(pitch) * (-t / 0.07).exp() + 0.3 * noise.next() * (-t / 0.003).exp()
            })
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Sounds {
    click: Handle<Chime>,
    slide: Handle<Chime>,
    puff: Handle<Chime>,
    splashes: Vec<(Voice, Handle<Chime>)>,
}

pub fn register(app: &mut App) {
    app.add_message::<Cue>().add_message::<Splash>();

    if app.is_plugin_added::<AudioPlugin>() {
        app.add_audio_source::<Chime>().add_systems(Startup, forge);
    }
}

fn forge(mut commands: Commands, mut chimes: ResMut<Assets<Chime>>) {
    commands.insert_resource(Sounds {
        click: chimes.add(Chime::new(click())),
        slide: chimes.add(Chime::new(slide())),
        puff: chimes.add(Chime::new(puff())),
        splashes: VOICES
            .into_iter()
            .map(|voice| (voice, chimes.add(Chime::new(splash(voice)))))
            .collect(),
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

pub fn splashes(
    mut commands: Commands,
    mut asked: MessageReader<Splash>,
    sounds: Option<Res<Sounds>>,
    options: Res<Options>,
    sounding: Query<&Sounding>,
) {
    let mut loudest: Vec<(Voice, f32)> = Vec::new();
    for splash in asked.read() {
        match loudest.iter_mut().find(|(voice, _)| *voice == splash.voice) {
            Some((_, volume)) => *volume = volume.max(splash.volume),
            None => loudest.push((splash.voice, splash.volume)),
        }
    }

    let Some(sounds) = sounds else { return };
    let base = effects(&options);
    if base <= 0.0 {
        return;
    }

    for (voice, volume) in loudest {
        let playing = sounding.iter().filter(|s| s.0 == voice).count();
        if playing >= SPLASH_VOICES || volume <= 0.0 {
            continue;
        }
        let Some((_, chime)) = sounds.splashes.iter().find(|(v, _)| *v == voice) else {
            continue;
        };
        commands.spawn((
            Sounding(voice),
            AudioPlayer(chime.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(base * volume)),
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
    fn every_landing_sound_is_short_quiet_and_clean() {
        for voice in VOICES {
            let wave = splash(voice);
            assert!(wave.len() < RATE as usize / 2, "{voice:?}");
            assert!(peak(&wave) <= 0.22 + 1e-4, "{voice:?}");
            assert!(peak(&wave) > 0.05, "{voice:?}");
            assert!(wave.iter().all(|sample| sample.is_finite()), "{voice:?}");
            assert!(
                wave[0].abs() < 0.01 && wave[wave.len() - 1].abs() < 0.01,
                "{voice:?}"
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
