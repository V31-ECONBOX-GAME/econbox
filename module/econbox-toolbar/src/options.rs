//! Every option the settings window offers, the values the player chose, and the file they live in.

use crate::{OpenWindow, Panel};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use econbox_art::{Glyph, Icon};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Interface,
    Video,
    Audio,
    Graphics,
    Other,
}

impl Section {
    pub fn title(self) -> &'static str {
        match self {
            Self::Interface => "Interface",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Graphics => "Graphics",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    Toggle(bool),
    Slider { min: i32, max: i32, default: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    Glyph(Glyph),
    Icon(Icon),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setting {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub section: Section,
    pub control: Control,
    pub mark: Mark,
    pub missing: Option<&'static str>,
}

const fn toggle(
    id: &'static str,
    name: &'static str,
    about: &'static str,
    section: Section,
    on: bool,
    mark: Mark,
    missing: Option<&'static str>,
) -> Setting {
    Setting {
        id,
        name,
        about,
        section,
        control: Control::Toggle(on),
        mark,
        missing,
    }
}

const fn slider(
    id: &'static str,
    name: &'static str,
    about: &'static str,
    section: Section,
    range: (i32, i32, i32),
    mark: Mark,
    missing: Option<&'static str>,
) -> Setting {
    Setting {
        id,
        name,
        about,
        section,
        control: Control::Slider {
            min: range.0,
            max: range.1,
            default: range.2,
        },
        mark,
        missing,
    }
}

use Mark::{Glyph as G, Icon as I};
use Section::*;

pub const SETTINGS: [Setting; 33] = [
    slider(
        "ui_size_main",
        "Bottom Power Bar Size",
        "How large the god power bar along the bottom is drawn.",
        Interface,
        (50, 150, 100),
        G(Glyph::Crosshair),
        None,
    ),
    slider(
        "ui_size_windows",
        "Window Size",
        "How large windows like this one are drawn.",
        Interface,
        (30, 115, 100),
        G(Glyph::List),
        None,
    ),
    slider(
        "ui_size_tooltips",
        "Tooltip Size",
        "How large the hover cards are drawn.",
        Interface,
        (30, 150, 100),
        G(Glyph::Info),
        None,
    ),
    slider(
        "ui_size_map_names",
        "Map Name Size",
        "How large the names written over the map are.",
        Interface,
        (30, 160, 70),
        G(Glyph::Tag),
        Some("map names"),
    ),
    slider(
        "ui_map_border_opacity",
        "Border Opacity",
        "How solid the borders between realms look.",
        Interface,
        (30, 100, 88),
        G(Glyph::Flag),
        Some("kingdom borders"),
    ),
    slider(
        "ui_map_border_brightness",
        "Border Brightness",
        "How bright the borders between realms look.",
        Interface,
        (60, 100, 100),
        G(Glyph::Bulb),
        Some("kingdom borders"),
    ),
    toggle(
        "fullscreen",
        "Fullscreen",
        "Fill the whole screen instead of a window.",
        Video,
        false,
        G(Glyph::Terminal),
        None,
    ),
    toggle(
        "vsync",
        "VSync",
        "Wait for the display before drawing each frame.",
        Video,
        false,
        G(Glyph::Swirl),
        None,
    ),
    toggle(
        "fps_lock_30",
        "30 FPS Lock",
        "Draw at most thirty frames a second to save power.",
        Video,
        false,
        I(Icon::Hourglass),
        None,
    ),
    slider(
        "volume_master_sound",
        "Master Volume",
        "Loudness of every sound.",
        Audio,
        (0, 100, 100),
        G(Glyph::Horn),
        None,
    ),
    slider(
        "volume_music",
        "Music Volume",
        "Loudness of the music.",
        Audio,
        (0, 100, 100),
        G(Glyph::Note),
        Some("music"),
    ),
    slider(
        "volume_sound_effects",
        "Sound Effects Volume",
        "Loudness of sounds from the world.",
        Audio,
        (0, 100, 100),
        G(Glyph::Bell),
        None,
    ),
    slider(
        "volume_ui",
        "Interface Volume",
        "Loudness of clicks and other interface sounds.",
        Audio,
        (0, 100, 100),
        G(Glyph::Bubbles),
        None,
    ),
    toggle(
        "fire",
        "Fire",
        "Show flames where things burn.",
        Graphics,
        true,
        I(Icon::Flame),
        Some("fire"),
    ),
    toggle(
        "vignette",
        "Vignette",
        "Darken the corners of the screen.",
        Graphics,
        true,
        G(Glyph::Eye),
        Some("screen effects"),
    ),
    toggle(
        "bloom",
        "Bloom",
        "Let bright things glow.",
        Graphics,
        true,
        I(Icon::Sun),
        Some("screen effects"),
    ),
    toggle(
        "smoke",
        "Smoke",
        "Show smoke rising from fires and chimneys.",
        Graphics,
        true,
        I(Icon::Cloud),
        Some("smoke"),
    ),
    toggle(
        "shadows",
        "Shadows",
        "Draw shadows under creatures and buildings.",
        Graphics,
        true,
        G(Glyph::Monolith),
        Some("shadows"),
    ),
    toggle(
        "tree_wind",
        "Tree Wind",
        "Let trees sway in the wind.",
        Graphics,
        true,
        G(Glyph::LivingPlant),
        Some("swaying trees"),
    ),
    toggle(
        "night_lights",
        "Night Lights",
        "Light windows and fires after dark.",
        Graphics,
        true,
        G(Glyph::Beacon),
        Some("day and night"),
    ),
    toggle(
        "minimap_transition_animation",
        "Map Mode Transition",
        "Animate the change between map views.",
        Graphics,
        true,
        G(Glyph::MapPen),
        Some("map modes"),
    ),
    toggle(
        "age_particles",
        "Age Particles",
        "Show the drifting particles of the current age.",
        Graphics,
        true,
        G(Glyph::Dust),
        Some("world ages"),
    ),
    slider(
        "age_overlay_effect",
        "Age Overlay",
        "Strength of the tint each age puts over the world.",
        Graphics,
        (0, 100, 100),
        G(Glyph::Layers),
        Some("world ages"),
    ),
    slider(
        "age_night_effect",
        "Age Night Effect",
        "How dark nights become in darker ages.",
        Graphics,
        (50, 100, 100),
        G(Glyph::Sleep),
        Some("world ages"),
    ),
    toggle(
        "autosaves",
        "Autosaves",
        "Save the world now and then on its own.",
        Other,
        true,
        G(Glyph::Disk),
        Some("saving"),
    ),
    toggle(
        "graphs",
        "Graphs",
        "Record history for the statistics graphs.",
        Other,
        true,
        G(Glyph::Chart),
        Some("statistics graphs"),
    ),
    toggle(
        "preload_windows",
        "Preload Windows",
        "Build windows ahead of time so they open faster.",
        Other,
        true,
        G(Glyph::Book),
        Some("asset preloading"),
    ),
    toggle(
        "preload_quantum_sprites",
        "Preload Effects",
        "Prepare effect sprites ahead of time.",
        Other,
        true,
        G(Glyph::Atom),
        Some("asset preloading"),
    ),
    toggle(
        "preload_buildings",
        "Preload Buildings",
        "Prepare building sprites ahead of time.",
        Other,
        true,
        G(Glyph::House),
        Some("asset preloading"),
    ),
    toggle(
        "preload_units",
        "Preload Units",
        "Prepare creature sprites ahead of time.",
        Other,
        true,
        G(Glyph::Twins),
        Some("asset preloading"),
    ),
    toggle(
        "cursor_lights",
        "Cursor Lights",
        "Let the pointer glow over the map at night.",
        Other,
        true,
        G(Glyph::Sunbeam),
        Some("cursor lights"),
    ),
    toggle(
        "tooltips",
        "Tooltips",
        "Show a card when the pointer rests on a button.",
        Other,
        true,
        G(Glyph::Question),
        None,
    ),
    toggle(
        "experimental",
        "Experimental Mode",
        "Turn on features that are still being tried out.",
        Other,
        false,
        I(Icon::Flask),
        Some("experimental features"),
    ),
];

pub fn setting(id: &str) -> Option<&'static Setting> {
    SETTINGS.iter().find(|setting| setting.id == id)
}

pub fn index(id: &str) -> usize {
    SETTINGS
        .iter()
        .position(|setting| setting.id == id)
        .unwrap_or_else(|| panic!("unknown option {id}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Int(i32),
}

impl Setting {
    pub fn default_value(&self) -> Value {
        match self.control {
            Control::Toggle(on) => Value::Bool(on),
            Control::Slider { default, .. } => Value::Int(default),
        }
    }

    pub fn clamp(&self, value: Value) -> Value {
        match (self.control, value) {
            (Control::Toggle(_), Value::Bool(on)) => Value::Bool(on),
            (Control::Slider { min, max, .. }, Value::Int(at)) => Value::Int(at.clamp(min, max)),
            _ => self.default_value(),
        }
    }

    pub fn counter(&self, value: Value) -> String {
        match value {
            Value::Int(at) => format!("{at}%"),
            Value::Bool(true) => "ON".to_string(),
            Value::Bool(false) => "OFF".to_string(),
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Options {
    values: Vec<Value>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            values: SETTINGS.iter().map(Setting::default_value).collect(),
        }
    }
}

impl Options {
    pub fn get(&self, id: &str) -> Value {
        self.values[index(id)]
    }

    pub fn on(&self, id: &str) -> bool {
        match self.get(id) {
            Value::Bool(on) => on,
            Value::Int(at) => at > 0,
        }
    }

    pub fn int(&self, id: &str) -> i32 {
        match self.get(id) {
            Value::Int(at) => at,
            Value::Bool(on) => i32::from(on),
        }
    }

    pub fn percent(&self, id: &str) -> f32 {
        self.int(id) as f32 / 100.0
    }

    pub fn set(&mut self, id: &str, value: Value) -> bool {
        let at = index(id);
        let value = SETTINGS[at].clamp(value);
        let moved = self.values[at] != value;
        self.values[at] = value;

        if moved {
            self.exclude(id, value);
        }

        moved
    }

    fn exclude(&mut self, id: &str, value: Value) {
        let other = match (id, value) {
            ("vsync", Value::Bool(true)) => "fps_lock_30",
            ("fps_lock_30", Value::Bool(true)) => "vsync",
            _ => return,
        };
        self.values[index(other)] = Value::Bool(false);
    }

    pub fn flip(&mut self, id: &str) -> bool {
        let on = self.on(id);
        self.set(id, Value::Bool(!on))
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn parse(text: &str) -> Self {
        let mut options = Self::default();

        for line in text.lines() {
            let Some((key, raw)) = line.split_once('=') else {
                continue;
            };
            let (key, raw) = (key.trim(), raw.trim());
            let Some(setting) = setting(key) else {
                continue;
            };

            let value = match setting.control {
                Control::Toggle(_) => raw.parse::<bool>().ok().map(Value::Bool),
                Control::Slider { .. } => raw.parse::<i32>().ok().map(Value::Int),
            };

            if let Some(value) = value {
                let at = index(key);
                options.values[at] = setting.clamp(value);
            }
        }

        options
    }

    pub fn text(&self) -> String {
        SETTINGS
            .iter()
            .zip(&self.values)
            .map(|(setting, value)| match value {
                Value::Bool(on) => format!("{} = {on}\n", setting.id),
                Value::Int(at) => format!("{} = {at}\n", setting.id),
            })
            .collect()
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsFile(pub Option<PathBuf>);

impl SettingsFile {
    pub fn load(&self) -> Option<Options> {
        let path = self.0.as_ref()?;
        let text = std::fs::read_to_string(path).ok()?;
        Some(Options::parse(&text))
    }

    pub fn save(&self, options: &Options) {
        let Some(path) = &self.0 else { return };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Err(error) = std::fs::write(path, options.text()) {
            warn!("could not save settings to {}: {error}", path.display());
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct Unsaved(pub bool);

pub fn track(options: Res<Options>, mut unsaved: ResMut<Unsaved>) {
    if options.is_changed() && !options.is_added() {
        unsaved.0 = true;
    }
}

pub fn save(
    open: Res<OpenWindow>,
    file: Res<SettingsFile>,
    options: Res<Options>,
    mut unsaved: ResMut<Unsaved>,
) {
    if unsaved.0 && open.0 != Some(Panel::Settings) {
        file.save(&options);
        unsaved.0 = false;
    }
}

pub fn frame_rate(options: &Options) -> Option<f64> {
    if options.on("vsync") {
        None
    } else if options.on("fps_lock_30") {
        Some(30.0)
    } else {
        Some(60.0)
    }
}

pub fn apply_display(
    options: Res<Options>,
    mut windows: Query<&mut Window>,
    mut seen: Local<Option<(bool, bool)>>,
) {
    let wanted = (options.on("fullscreen"), options.on("vsync"));
    if *seen == Some(wanted) {
        return;
    }
    let first = seen.is_none();
    *seen = Some(wanted);

    for mut window in &mut windows {
        let mode = if wanted.0 {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        if (!first || wanted.0) && window.mode != mode {
            window.mode = mode;
        }

        let present = if wanted.1 {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != present {
            window.present_mode = present;
        }
    }
}

pub fn limit_frames(
    options: Res<Options>,
    windows: Query<(), With<Window>>,
    mut last: Local<Option<Instant>>,
) {
    let Some(rate) = frame_rate(&options).filter(|_| !windows.is_empty()) else {
        *last = None;
        return;
    };

    let frame = Duration::from_secs_f64(1.0 / rate);
    if let Some(previous) = *last {
        let spent = previous.elapsed();
        if spent < frame {
            std::thread::sleep(frame - spent);
        }
    }
    *last = Some(Instant::now());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_cap_follows_vsync_and_the_thirty_lock() {
        let mut options = Options::default();
        assert_eq!(frame_rate(&options), Some(60.0));
        options.set("fps_lock_30", Value::Bool(true));
        assert_eq!(frame_rate(&options), Some(30.0));
        options.set("vsync", Value::Bool(true));
        assert_eq!(frame_rate(&options), None);
    }

    #[test]
    fn every_option_has_a_unique_id_and_a_sane_range() {
        for (at, setting) in SETTINGS.iter().enumerate() {
            assert_eq!(index(setting.id), at, "{}", setting.id);
            if let Control::Slider { min, max, default } = setting.control {
                assert!(
                    min < max && (min..=max).contains(&default),
                    "{}",
                    setting.id
                );
            }
        }
    }

    #[test]
    fn values_are_clamped_and_survive_a_round_trip() {
        let mut options = Options::default();
        assert!(options.set("ui_size_main", Value::Int(400)));
        assert_eq!(options.int("ui_size_main"), 150);
        options.set("fire", Value::Bool(false));

        let back = Options::parse(&options.text());
        assert_eq!(back, options);
    }

    #[test]
    fn vsync_and_the_frame_lock_exclude_each_other() {
        let mut options = Options::default();
        options.set("fps_lock_30", Value::Bool(true));
        options.set("vsync", Value::Bool(true));
        assert!(!options.on("fps_lock_30"));

        options.flip("fps_lock_30");
        assert!(!options.on("vsync"));
    }

    #[test]
    fn unknown_or_broken_lines_keep_the_defaults() {
        let options = Options::parse("nonsense = 3\nfire = maybe\nui_size_windows = 50\n");
        assert!(options.on("fire"));
        assert_eq!(options.int("ui_size_windows"), 50);
    }

    #[test]
    fn sliders_count_in_percent() {
        let main = setting("ui_size_main").unwrap();
        assert_eq!(main.counter(Value::Int(120)), "120%");
    }
}
