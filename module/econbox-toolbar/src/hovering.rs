//! The dimmed backdrop behind open windows and the icons that hover in it.

use crate::art::Kit;
use crate::bar::px;
use crate::motion::out_cubic;
use crate::options::Options;
use crate::size::UiLayer;
use crate::windows::{Shown, screen_units};
use crate::{OpenWindow, Panel};
use bevy::picking::events::PointerClick;
use bevy::prelude::*;
use std::f32::consts::PI;

pub const PLACES: usize = 68;
pub const DIM: f32 = 0.4667;
pub const GAMMA: f32 = 2.2;
pub const DIM_SPEED: f32 = 5.0;
pub const FADE_TIME: f32 = 0.2;
pub const ICON: f32 = 28.0;
pub const BOB: f32 = 3.0;
pub const FALL_TIME: f32 = 0.6;
pub const SPIN_TIME: f32 = 1.0;

#[derive(Component)]
pub struct Dim;

#[derive(Component)]
pub struct Drift;

#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Floater {
    pub from: Vec2,
    pub to: Vec2,
    pub travel: f32,
    pub travel_time: f32,
    pub scale: f32,
    pub angle: f32,
    pub bob_clock: f32,
    pub bob_time: f32,
    pub bob_from: f32,
    pub fall: Option<f32>,
    pub spin: f32,
    pub jitter: f32,
    pub live: bool,
}

impl Floater {
    pub fn place(&self) -> Vec2 {
        let t = if self.travel_time > 0.0 {
            (self.travel / self.travel_time).min(1.0)
        } else {
            1.0
        };
        let eased = 1.0 - (1.0 - t) * (1.0 - t);
        self.from.lerp(self.to, eased)
    }

    pub fn bob(&self) -> f32 {
        let phase = (self.bob_clock / self.bob_time.max(f32::EPSILON)) % 2.0;
        let leg = if phase < 1.0 { phase } else { 2.0 - phase };
        let eased = if leg < 0.5 {
            2.0 * leg * leg
        } else {
            1.0 - (-2.0 * leg + 2.0).powi(2) * 0.5
        };
        self.bob_from + BOB * eased
    }

    pub fn shade(&self) -> f32 {
        self.scale
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Drifting {
    pub alpha: f32,
    pub showing: bool,
    seed: u64,
}

impl Default for Drifting {
    fn default() -> Self {
        Self {
            alpha: 0.0,
            showing: false,
            seed: 0x9e37_79b9_7f4a_7c15,
        }
    }
}

impl Drifting {
    pub fn roll(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.roll()
    }
}

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Dim,
        Pickable::IGNORE,
        GlobalZIndex(400),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));

    commands
        .spawn((
            Drift,
            UiLayer::Windows,
            Pickable::IGNORE,
            GlobalZIndex(450),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                top: px(0.0),
                bottom: px(0.0),
                ..default()
            },
        ))
        .with_children(|drift| {
            for _ in 0..PLACES {
                drift.spawn((
                    Floater::default(),
                    UiTransform::default(),
                    Visibility::Hidden,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(ICON),
                        height: px(ICON),
                        ..default()
                    },
                    ImageNode::default(),
                ));
            }
        });
}

pub fn dim(
    time: Res<Time>,
    open: Res<OpenWindow>,
    mut dims: Query<&mut BackgroundColor, With<Dim>>,
    mut level: Local<f32>,
) {
    let step = time.delta_secs() * DIM_SPEED;
    let wanted = if open.0.is_some() {
        (*level + step).min(1.0)
    } else {
        (*level - step).max(0.0)
    };

    for mut color in &mut dims {
        let shade = Color::srgba(0.0, 0.0, 0.0, gamma_alpha(DIM * wanted));
        if color.0 != shade {
            color.0 = shade;
        }
    }

    *level = wanted;
}

pub fn gamma_alpha(alpha: f32) -> f32 {
    1.0 - (1.0 - alpha.clamp(0.0, 1.0)).powf(GAMMA)
}

fn pictures(kit: &Kit, panel: Panel) -> Vec<Handle<Image>> {
    match panel {
        Panel::Brushes => {
            let named = [
                "circ_1", "circ_10", "circ_15", "circ_2", "circ_5", "sqr_1", "sqr_10", "sqr_15",
                "sqr_2", "sqr_5",
            ];
            econbox_brush::PRESETS
                .iter()
                .enumerate()
                .filter(|(_, preset)| named.contains(&preset.id))
                .map(|(at, _)| kit.brushes[at].0.clone())
                .collect()
        }
        Panel::Other | Panel::Settings => kit
            .glyphs
            .get(&panel.glyph())
            .cloned()
            .into_iter()
            .collect(),
    }
}

pub fn scatter(
    mut shown: MessageReader<Shown>,
    windows: Query<&Window>,
    options: Res<Options>,
    kit: Option<Res<Kit>>,
    mut drifting: ResMut<Drifting>,
    mut floaters: Query<(&mut Floater, &mut ImageNode, &mut Visibility)>,
) {
    let Some(Shown(panel)) = shown.read().last().copied() else {
        return;
    };
    let Some(kit) = kit else { return };

    let pool = pictures(&kit, panel);
    if pool.is_empty() {
        return;
    }

    let screen = screen_units(&windows, &options);
    let gather = drifting.roll() < 0.5;
    let mut angle = drifting.between(0.0, 360.0);
    drifting.showing = true;

    for (mut floater, mut image, mut visibility) in &mut floaters {
        let from = if gather || !floater.live {
            screen * 0.5
        } else {
            floater.place()
        };
        let to = Vec2::new(
            drifting.between(0.0, screen.x),
            drifting.between(0.0, screen.y),
        );
        let scale = drifting.between(0.4, 1.0);
        angle += drifting.between(20.0, 130.0);
        let pick = ((drifting.roll() * pool.len() as f32) as usize).min(pool.len() - 1);

        *floater = Floater {
            from,
            to,
            travel: 0.0,
            travel_time: drifting.between(0.15, 0.35),
            scale,
            angle,
            bob_clock: if drifting.roll() < 0.5 { 0.0 } else { 1.0 },
            bob_time: drifting.between(1.0, 1.5),
            bob_from: drifting.between(-2.0, 2.0),
            fall: None,
            spin: 0.0,
            jitter: 0.0,
            live: true,
        };
        image.image = pool[pick].clone();
        visibility.set_if_neq(Visibility::Inherited);
    }
}

pub fn drift(
    time: Res<Time>,
    open: Res<OpenWindow>,
    windows: Query<&Window>,
    options: Res<Options>,
    mut drifting: ResMut<Drifting>,
    mut roots: Query<&mut Visibility, (With<Drift>, Without<Floater>)>,
    mut floaters: Query<(
        &mut Floater,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
        &mut Visibility,
    )>,
) {
    let dt = time.delta_secs();
    let fading_in = open.0.is_some() && drifting.showing;
    let step = dt / FADE_TIME;
    drifting.alpha = if fading_in {
        (drifting.alpha + step).min(1.0)
    } else {
        (drifting.alpha - step).max(0.0)
    };

    if open.0.is_none() && drifting.showing {
        drifting.showing = false;
        for (mut floater, _, _, _, mut visibility) in &mut floaters {
            floater.live = false;
            visibility.set_if_neq(Visibility::Hidden);
        }
    }

    for mut visibility in &mut roots {
        visibility.set_if_neq(if drifting.alpha > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }

    if drifting.alpha <= 0.0 {
        return;
    }

    let fall_by = screen_units(&windows, &options).y;
    let alpha = drifting.alpha;

    for (mut floater, mut node, mut transform, mut image, mut visibility) in &mut floaters {
        if !floater.live {
            continue;
        }

        floater.travel += dt;
        let place = floater.place();
        let (mut offset, mut angle) = (Vec2::new(0.0, floater.bob()), floater.angle);

        if let Some(clock) = floater.fall {
            let clock = clock + dt;
            floater.fall = Some(clock);
            let t = (clock / FALL_TIME).min(1.0);
            offset = Vec2::new(floater.jitter * t, -fall_by * t * t);
            angle += out_cubic((clock / SPIN_TIME).min(1.0)) * floater.spin;

            if clock >= FALL_TIME {
                floater.live = false;
                visibility.set_if_neq(Visibility::Hidden);
                continue;
            }
        } else {
            floater.bob_clock += dt;
        }

        let left = px(place.x - ICON * 0.5);
        let bottom = px(place.y - ICON * 0.5);
        if node.left != left || node.bottom != bottom {
            node.left = left;
            node.bottom = bottom;
        }

        transform.translation = Val2::px(offset.x, -offset.y);
        transform.scale = Vec2::splat(floater.scale);
        transform.rotation = Rot2::radians(-angle * PI / 180.0);

        let gray = floater.shade();
        image.color = Color::srgba(gray, gray, gray, alpha);
    }
}

pub fn drop_one(floater: &mut Floater, drifting: &mut Drifting) {
    if floater.fall.is_some() || !floater.live {
        return;
    }

    let spin = drifting.between(90.0, 180.0);
    floater.spin = if drifting.roll() < 0.5 { spin } else { -spin };
    floater.jitter = drifting.between(-4.0, 4.0);
    floater.angle = floater.angle.rem_euclid(360.0);
    floater.fall = Some(0.0);
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut drifting: ResMut<Drifting>,
    mut floaters: Query<&mut Floater>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    if let Ok(mut floater) = floaters.get_mut(click.entity) {
        click.propagate(false);
        drop_one(&mut floater, &mut drifting);
    }
}

pub fn drop_on_resize(
    options: Res<Options>,
    mut seen: Local<Option<i32>>,
    mut drifting: ResMut<Drifting>,
    mut floaters: Query<&mut Floater>,
) {
    let size = options.int("ui_size_windows");

    if seen.is_some_and(|last| last != size) {
        for mut floater in &mut floaters {
            drop_one(&mut floater, &mut drifting);
        }
    }

    *seen = Some(size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_floater_eases_to_its_place_and_bobs_three_units() {
        let mut floater = Floater {
            from: Vec2::ZERO,
            to: Vec2::new(100.0, 50.0),
            travel_time: 0.2,
            bob_time: 1.0,
            scale: 0.5,
            live: true,
            ..default()
        };
        assert_eq!(floater.place(), Vec2::ZERO);
        floater.travel = 0.2;
        assert_eq!(floater.place(), Vec2::new(100.0, 50.0));

        assert_eq!(floater.bob(), 0.0);
        floater.bob_clock = 1.0;
        assert!((floater.bob() - BOB).abs() < 1e-5);
        floater.bob_clock = 2.0;
        assert!(floater.bob().abs() < 1e-5);
    }

    #[test]
    fn the_dim_darkens_like_a_gamma_space_blend() {
        let kept = 1.0 - gamma_alpha(DIM);
        assert!((kept.powf(1.0 / GAMMA) - (1.0 - DIM)).abs() < 1e-4);
        assert_eq!(gamma_alpha(0.0), 0.0);
        assert_eq!(gamma_alpha(1.0), 1.0);
    }

    #[test]
    fn the_dice_stay_inside_their_range() {
        let mut drifting = Drifting::default();
        for _ in 0..1000 {
            let roll = drifting.between(0.4, 1.0);
            assert!((0.4..1.0).contains(&roll));
        }
    }

    #[test]
    fn a_dropped_icon_spins_between_a_quarter_and_a_half_turn() {
        let mut drifting = Drifting::default();
        let mut floater = Floater {
            live: true,
            ..default()
        };
        drop_one(&mut floater, &mut drifting);
        assert_eq!(floater.fall, Some(0.0));
        assert!((90.0..=180.0).contains(&floater.spin.abs()));
        assert!(floater.jitter.abs() <= 4.0);
    }
}
