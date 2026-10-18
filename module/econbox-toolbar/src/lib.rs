//! The god-power bar: tabs of buttons along the bottom, the side controls, and the keys that drive them.

mod art;
mod bar;
mod brushes;
pub mod catalog;
mod keys;
pub mod layout;
mod motion;
mod panels;
mod pointer;
mod press;
mod side;
pub mod size;
mod sound;
pub mod strip;
mod tabs;
mod tips;

use bevy::app::PropagateSet;
use bevy::ecs::system::SystemParam;
use bevy::picking::PickingSystems;
use bevy::prelude::*;
use bevy::ui::ComputedUiRenderTargetInfo;
use catalog::{Act, Def, Kind, TabId};
use econbox_brush::{Brush, ForcedBrush, Shape, Tool, named};
use econbox_clock::WorldClock;

pub use catalog::{find, home_of};
pub use size::{BarSize, KeepSize};
pub use sound::Cue;
pub use strip::StripOrder;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shown(pub TabId);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Chosen(pub Option<&'static str>);

impl Chosen {
    pub fn def(&self) -> Option<&'static Def> {
        self.0.and_then(find)
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiHidden(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Brushes,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpenWindow(pub Option<Panel>);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpeedRow(pub bool);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Slots(pub [Option<&'static str>; 10]);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyCursor(pub Option<&'static str>);

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct Notice(pub String);

#[derive(SystemParam)]
pub struct Controls<'w> {
    pub chosen: ResMut<'w, Chosen>,
    pub brush: ResMut<'w, Brush>,
    pub forced: ResMut<'w, ForcedBrush>,
    pub clock: ResMut<'w, WorldClock>,
    pub hidden: ResMut<'w, UiHidden>,
    pub speeds: ResMut<'w, SpeedRow>,
    pub notices: MessageWriter<'w, Notice>,
}

impl Controls<'_> {
    pub fn press(&mut self, def: &'static Def) {
        match (def.kind, def.act) {
            (_, Act::Planned(system)) => {
                self.notices.write(Notice(format!(
                    "{}\nNot available yet: needs {system}.",
                    def.name
                )));
            }
            (Kind::Action, Act::Pause) => {
                self.clock.toggle();
                let said = if self.clock.paused {
                    "Paused"
                } else {
                    "Unpaused"
                };
                self.notices.write(Notice(said.to_string()));
            }
            (Kind::Action, Act::HideUi) => {
                self.hidden.0 = true;
            }
            (Kind::Select | Kind::Spawn, _) => {
                if self.chosen.0 == Some(def.id) {
                    self.deselect();
                } else {
                    self.select(def);
                }
            }
            _ => {}
        }
    }

    pub fn select(&mut self, def: &'static Def) {
        if !def.selectable() {
            return;
        }

        self.chosen.0 = Some(def.id);
        self.brush.tool = def.tool().unwrap_or(Tool::Inspect);
        self.forced.0 = forced_for(def);
        self.speeds.0 = false;
        self.notices
            .write(Notice(format!("{}\n{}", def.name, def.about)));
    }

    pub fn deselect(&mut self) {
        if self.chosen.0.is_some() {
            self.chosen.0 = None;
        }

        if self.brush.tool != Tool::Inspect {
            self.brush.tool = Tool::Inspect;
        }

        if self.forced.0.is_some() {
            self.forced.0 = None;
        }

        self.speeds.0 = false;
    }
}

pub fn forced_for(def: &Def) -> Option<(Shape, u32)> {
    if let Some(Tool::Power(power)) = def.tool()
        && !def.sizes
        && let Some(reach) = power.reach()
    {
        return Some((Shape::Round, reach));
    }

    def.brush.and_then(named)
}

pub fn canvas_scale(window: Vec2, size: BarSize) -> f32 {
    (window.x / layout::CANVAS_WIDTH)
        .min(window.y / (layout::CANVAS_HEIGHT * size.factor()))
        .max(0.01)
}

fn fit(windows: Query<&Window>, size: Res<BarSize>, mut scale: ResMut<UiScale>) {
    let Some(window) = windows.iter().next() else {
        return;
    };

    let wanted = canvas_scale(Vec2::new(window.width(), window.height()), *size);

    if (scale.0 - wanted).abs() > f32::EPSILON {
        scale.0 = wanted;
    }
}

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .init_resource::<Chosen>()
            .init_resource::<UiHidden>()
            .init_resource::<OpenWindow>()
            .init_resource::<SpeedRow>()
            .init_resource::<Slots>()
            .init_resource::<KeyCursor>()
            .init_resource::<bar::Scroll>()
            .init_resource::<tips::Hover>()
            .init_resource::<pointer::Grace>()
            .init_resource::<BarSize>()
            .init_resource::<size::Preview>()
            .init_resource::<StripOrder>()
            .init_resource::<strip::StripDrag>()
            .add_message::<Notice>()
            .add_observer(press::on_click)
            .add_observer(press::on_press)
            .add_observer(side::on_click)
            .add_observer(brushes::on_click)
            .add_systems(
                Startup,
                (
                    art::forge,
                    bar::spawn,
                    side::spawn,
                    brushes::spawn,
                    tips::spawn,
                    pointer::spawn,
                )
                    .chain(),
            )
            .add_systems(PreUpdate, pointer::over_ui.after(PickingSystems::Hover))
            .add_systems(
                Update,
                (
                    (
                        size::resize,
                        fit,
                        size::preview,
                        keys::hotkeys,
                        keys::wheel,
                        bar::drag,
                        strip::drag,
                        bar::switch,
                        bar::arrows,
                    )
                        .chain(),
                    (
                        side::show,
                        brushes::show,
                        press::dress,
                        press::hover,
                        panels::refresh,
                        tips::tooltip,
                        tips::notice,
                        pointer::follow,
                        pointer::footprint,
                        size::fade,
                        sound::play,
                    )
                        .chain(),
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (motion::movers, motion::pops, motion::tilts).before(bevy::ui::UiSystems::Layout),
            )
            .add_systems(
                PostUpdate,
                size::keep
                    .after(bevy::ui::update::propagate_ui_target_cameras)
                    .before(PropagateSet::<ComputedUiRenderTargetInfo>::default()),
            );

        sound::register(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_canvas_is_five_hundred_units_tall_on_a_wide_window() {
        let size = BarSize::default();
        assert_eq!(canvas_scale(Vec2::new(2000.0, 1000.0), size), 2.0);
        assert_eq!(canvas_scale(Vec2::new(1280.0, 720.0), size), 720.0 / 500.0);
    }

    #[test]
    fn the_bar_size_scales_the_reference_height_by_two_minus_its_share() {
        let wide = Vec2::new(4000.0, 1000.0);

        assert_eq!(canvas_scale(wide, BarSize::new(150)), 1000.0 / 250.0);
        assert_eq!(canvas_scale(wide, BarSize::new(50)), 1000.0 / 750.0);
        assert!((canvas_scale(wide, BarSize::new(120)) - 1000.0 / 400.0).abs() < 1e-4);
        assert_eq!(canvas_scale(wide, BarSize::new(500)), 1000.0 / 250.0);
    }

    #[test]
    fn strikes_keep_their_own_size_and_tiles_follow_the_brush() {
        assert_eq!(
            forced_for(find("lightning").unwrap()),
            Some((Shape::Round, 2))
        );
        assert_eq!(
            forced_for(find("volcano").unwrap()),
            Some((Shape::Round, 5))
        );
        assert_eq!(forced_for(find("tile_sand").unwrap()), None);
        assert_eq!(forced_for(find("bomb").unwrap()), None);
        assert_eq!(
            forced_for(find("wall_order").unwrap()),
            Some((Shape::Square, 0))
        );
    }

    #[test]
    fn a_narrow_window_is_limited_by_its_width() {
        assert_eq!(
            canvas_scale(Vec2::new(285.0, 2000.0), BarSize::default()),
            1.0
        );
        assert_eq!(
            canvas_scale(Vec2::new(285.0, 2000.0), BarSize::new(150)),
            1.0
        );
    }
}
