//! The bottom power bar size option, the faint preview of the bar after it changes, and the scale of each interface layer.

use crate::bar::BarRoot;
use crate::keys::ctrl;
use crate::motion::Mover;
use crate::options::{Options, Value};
use crate::{Notice, OpenWindow, UiHidden, canvas_scale};
use bevy::app::Propagate;
use bevy::prelude::*;
use bevy::ui::ComputedUiRenderTargetInfo;

pub const MIN_SIZE: u32 = 50;
pub const MAX_SIZE: u32 = 150;
pub const DEFAULT_SIZE: u32 = 100;
pub const SIZE_STEP: i32 = 10;
pub const PREVIEW: f32 = 1.0;
pub const GHOST: f32 = 0.5;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarSize(u32);

impl Default for BarSize {
    fn default() -> Self {
        Self(DEFAULT_SIZE)
    }
}

impl BarSize {
    pub fn new(percent: u32) -> Self {
        Self(percent.clamp(MIN_SIZE, MAX_SIZE))
    }

    pub fn percent(self) -> u32 {
        self.0
    }

    pub fn factor(self) -> f32 {
        2.0 - self.0 as f32 / 100.0
    }

    pub fn step(&mut self, by: i32) -> bool {
        let next = Self::new(self.0.saturating_add_signed(by));
        let moved = next != *self;
        *self = next;
        moved
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLayer {
    Windows,
    Tooltips,
    Fixed,
}

pub const WINDOW_CANVAS_WIDTH: f32 = 285.0;
pub const WINDOW_CANVAS_HEIGHT: f32 = 420.0;

pub fn window_canvas_scale(window: Vec2, percent: i32) -> f32 {
    let factor = 2.0 - percent as f32 / 100.0;
    (window.x / WINDOW_CANVAS_WIDTH)
        .min(window.y / (WINDOW_CANVAS_HEIGHT * factor))
        .max(0.01)
}

pub fn kept_scale(window: Vec2) -> f32 {
    canvas_scale(window, BarSize::default())
}

pub fn layer_scale(layer: UiLayer, window: Vec2, options: &Options) -> f32 {
    match layer {
        UiLayer::Windows => window_canvas_scale(window, options.int("ui_size_windows")),
        UiLayer::Tooltips => window_canvas_scale(window, options.int("ui_size_tooltips")),
        UiLayer::Fixed => kept_scale(window),
    }
}

pub fn compensation(layer_scale: f32, bar_scale: f32) -> f32 {
    layer_scale / bar_scale
}

pub fn window_size(window: &Window) -> Vec2 {
    Vec2::new(window.width(), window.height())
}

fn rescale(target: &mut ComputedUiRenderTargetInfo, by: f32) {
    if let Some(factor) = target.get_field_mut::<f32>("scale_factor") {
        *factor *= by;
    }
}

pub fn keep(
    windows: Query<&Window>,
    scale: Res<UiScale>,
    options: Res<Options>,
    mut roots: Query<(&UiLayer, &mut Propagate<ComputedUiRenderTargetInfo>)>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let size = window_size(window);

    for (layer, mut target) in &mut roots {
        let by = compensation(layer_scale(*layer, size, &options), scale.0);

        if target.is_changed() && (by - 1.0).abs() > f32::EPSILON {
            rescale(&mut target.0, by);
        }
    }
}

pub fn follow_options(
    mut options: ResMut<Options>,
    mut size: ResMut<BarSize>,
    mut seen: Local<Option<(i32, u32)>>,
) {
    let option = options.int("ui_size_main");
    let bar = size.percent();

    match *seen {
        Some((last, _)) if option != last => *size = BarSize::new(option.max(0) as u32),
        Some((_, last)) if bar != last => {
            options.set("ui_size_main", Value::Int(bar as i32));
        }
        None if option != bar as i32 => {
            options.set("ui_size_main", Value::Int(bar as i32));
        }
        _ => {}
    }

    *seen = Some((options.int("ui_size_main"), size.percent()));
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Preview {
    left: f32,
    pub ghost: bool,
    pub faded: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Faded {
    base: f32,
    shown: f32,
}

pub fn resize(
    keys: Res<ButtonInput<KeyCode>>,
    mut size: ResMut<BarSize>,
    mut notices: MessageWriter<Notice>,
) {
    if !ctrl(&keys) {
        return;
    }

    let by = if keys.any_just_pressed([KeyCode::Equal, KeyCode::NumpadAdd]) {
        SIZE_STEP
    } else if keys.any_just_pressed([KeyCode::Minus, KeyCode::NumpadSubtract]) {
        -SIZE_STEP
    } else {
        return;
    };

    if size.step(by) {
        notices.write(Notice(format!(
            "Bottom Power Bar Size: {}%",
            size.percent()
        )));
    }
}

pub fn preview(
    time: Res<Time>,
    size: Res<BarSize>,
    open: Res<OpenWindow>,
    hidden: Res<UiHidden>,
    mut preview: ResMut<Preview>,
    bars: Query<&Mover, With<BarRoot>>,
) {
    let away = open.0.is_some() || hidden.0;

    if !away {
        if *preview != Preview::default() {
            *preview = Preview::default();
        }
        return;
    }

    if size.is_changed() && !size.is_added() {
        preview.left = PREVIEW;
        preview.faded = true;
    }

    preview.left = (preview.left - time.delta_secs()).max(0.0);
    preview.ghost = preview.left > 0.0;

    if preview.faded && !preview.ghost && bars.iter().all(|mover| !mover.visible && mover.settled())
    {
        preview.faded = false;
    }
}

fn shade(color: &mut Color, faded: Option<&mut Faded>, on: bool) -> Option<Faded> {
    let alpha = color.alpha();

    match (on, faded) {
        (true, None) => {
            let mark = Faded {
                base: alpha,
                shown: alpha * GHOST,
            };
            color.set_alpha(mark.shown);
            Some(mark)
        }
        (true, Some(mark)) if alpha != mark.shown => {
            mark.base = alpha;
            mark.shown = alpha * GHOST;
            color.set_alpha(mark.shown);
            None
        }
        (false, Some(mark)) if alpha == mark.shown => {
            color.set_alpha(mark.base);
            None
        }
        _ => None,
    }
}

pub fn fade(
    mut commands: Commands,
    preview: Res<Preview>,
    bars: Query<Entity, With<BarRoot>>,
    tree: Query<&Children>,
    mut images: Query<(&mut ImageNode, Option<&mut Faded>)>,
    mut texts: Query<(&mut TextColor, Option<&mut Faded>), Without<ImageNode>>,
    mut was: Local<bool>,
) {
    let on = preview.faded;

    if !on && !*was {
        return;
    }

    *was = on;

    for bar in &bars {
        for entity in tree.iter_descendants(bar) {
            let mark = if let Ok((mut image, mut faded)) = images.get_mut(entity) {
                let marked = faded.is_some();
                (shade(&mut image.color, faded.as_deref_mut(), on), marked)
            } else if let Ok((mut text, mut faded)) = texts.get_mut(entity) {
                let marked = faded.is_some();
                (shade(&mut text.0, faded.as_deref_mut(), on), marked)
            } else {
                continue;
            };

            match mark {
                (Some(fresh), _) => {
                    commands.entity(entity).insert(fresh);
                }
                (None, true) if !on => {
                    commands.entity(entity).remove::<Faded>();
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_size_stays_between_half_and_one_and_a_half() {
        assert_eq!(BarSize::new(10).percent(), MIN_SIZE);
        assert_eq!(BarSize::new(400).percent(), MAX_SIZE);

        let mut size = BarSize::default();
        assert!(size.step(SIZE_STEP));
        assert_eq!(size.percent(), 110);

        for _ in 0..20 {
            size.step(-SIZE_STEP);
        }
        assert_eq!(size.percent(), MIN_SIZE);
        assert!(!size.step(-SIZE_STEP));
    }

    #[test]
    fn the_reference_height_shrinks_as_the_bar_grows() {
        assert_eq!(BarSize::default().factor(), 1.0);
        assert_eq!(BarSize::new(MAX_SIZE).factor(), 0.5);
        assert_eq!(BarSize::new(MIN_SIZE).factor(), 1.5);
        assert!((BarSize::new(120).factor() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn each_layer_undoes_the_bar_size_and_lands_on_its_own_scale() {
        let wide = Vec2::new(1280.0, 720.0);
        let narrow = Vec2::new(285.0, 2000.0);
        let options = Options::default();

        for window in [wide, narrow, Vec2::new(600.0, 900.0)] {
            for percent in (MIN_SIZE..=MAX_SIZE).step_by(SIZE_STEP as usize) {
                let bar = canvas_scale(window, BarSize::new(percent));
                for layer in [UiLayer::Windows, UiLayer::Tooltips, UiLayer::Fixed] {
                    let wanted = layer_scale(layer, window, &options);
                    let kept = bar * compensation(wanted, bar);
                    assert!((kept - wanted).abs() < 1e-5, "{window} {percent} {layer:?}");
                }
            }
        }

        let bar = |percent| canvas_scale(wide, BarSize::new(percent));
        let fixed = kept_scale(wide);
        assert!((compensation(fixed, bar(MIN_SIZE)) - 1.5).abs() < 1e-5);
        assert!((compensation(fixed, bar(MAX_SIZE)) - 0.5).abs() < 1e-5);
        assert_eq!(compensation(fixed, bar(DEFAULT_SIZE)), 1.0);
        assert_eq!(kept_scale(wide), 720.0 / 500.0);
    }

    #[test]
    fn windows_and_tooltips_follow_their_own_size_options() {
        let wide = Vec2::new(1280.0, 720.0);
        let mut options = Options::default();
        assert_eq!(layer_scale(UiLayer::Windows, wide, &options), 720.0 / 420.0);

        options.set("ui_size_windows", Value::Int(50));
        options.set("ui_size_tooltips", Value::Int(150));
        assert!((layer_scale(UiLayer::Windows, wide, &options) - 720.0 / 630.0).abs() < 1e-5);
        assert!((layer_scale(UiLayer::Tooltips, wide, &options) - 720.0 / 210.0).abs() < 1e-5);
        assert_eq!(window_canvas_scale(Vec2::new(285.0, 2000.0), 100), 1.0);
    }

    #[test]
    fn a_rescaled_target_keeps_its_viewport_and_multiplies_its_scale_factor() {
        let mut target = ComputedUiRenderTargetInfo::default();
        rescale(&mut target, 1.5);
        assert_eq!(target.scale_factor(), 1.5);

        rescale(&mut target, 0.5);
        assert_eq!(target.scale_factor(), 0.75);
        assert_eq!(target.physical_size(), UVec2::ZERO);
    }

    #[test]
    fn a_ghost_halves_alpha_and_gives_it_back() {
        let mut color = Color::srgba(1.0, 1.0, 1.0, 0.8);
        let mut mark = shade(&mut color, None, true).expect("a fresh mark");
        assert!((color.alpha() - 0.4).abs() < 1e-6);

        assert_eq!(shade(&mut color, Some(&mut mark), true), None);
        assert!((color.alpha() - 0.4).abs() < 1e-6);

        color.set_alpha(0.9);
        shade(&mut color, Some(&mut mark), true);
        assert!((color.alpha() - 0.45).abs() < 1e-6);

        shade(&mut color, Some(&mut mark), false);
        assert!((color.alpha() - 0.9).abs() < 1e-6);
    }
}
