//! The hover card that names a button, and the line of text that rises above the bar.

use crate::art::{Kit, sliced};
use crate::bar::{Scroll, cursor_canvas, px};
use crate::catalog::Def;
use crate::options::Options;
use crate::size::{UiLayer, layer_scale, window_size};
use crate::{Notice, OpenWindow, UiHidden};
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::text::{Justify, LineBreak};

const WIDTH: f32 = 113.0;
const TEXT_WIDTH: f32 = 104.0;
const HEAD: f32 = 21.6;
const MARGIN: f32 = 21.0;
const MIN_SHOWN: f32 = 0.1;
const WHEEL_QUIET: f32 = 0.06;
const LEAVE_DELAY: f32 = 0.08;
const LEAVE_DRIFT: f32 = 10.0;
const NAME_SIZE: f32 = 8.0;
const BODY_SIZE: f32 = 6.0;

pub const TITLE: Color = Color::srgb(1.0, 0.6073, 0.1103);
pub const BODY: Color = Color::WHITE;
pub const FOOT: Color = Color::srgb(0.66, 0.66, 0.66);
pub const WARN: Color = Color::srgb(0.98, 0.42, 0.33);

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Tip {
    pub title: String,
    pub body: String,
    pub foot: Option<String>,
    pub warn: bool,
}

impl Tip {
    pub fn of(def: &Def) -> Self {
        let (foot, warn) = match def.missing() {
            Some(system) => (Some(format!("Not available yet: needs {system}.")), true),
            None => (def.hint.map(str::to_string), false),
        };

        Self {
            title: def.name.to_string(),
            body: def.about.to_string(),
            foot,
            warn,
        }
    }

    pub fn plain(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
            foot: None,
            warn: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct Hover {
    pub target: Option<Entity>,
    pub content: Option<Tip>,
    pub pending: Option<(Entity, Tip)>,
    pub visible: bool,
    wheel_quiet: f32,
    shown_for: f32,
    leaving: Option<(f32, Vec2)>,
    fresh: bool,
}

impl Hover {
    pub fn enter(&mut self, target: Entity, content: Tip) {
        self.target = Some(target);
        self.content = Some(content);
        self.visible = true;
        self.shown_for = 0.0;
        self.leaving = None;
        self.fresh = true;
    }

    pub fn leave(&mut self) {
        if self.leaving.is_none() {
            self.leaving = Some((0.0, Vec2::ZERO));
        }
    }

    pub fn wheel_quiet(&self) -> f32 {
        self.wheel_quiet
    }

    pub fn hide(&mut self) {
        self.target = None;
        self.visible = false;
        self.leaving = None;
    }
}

#[derive(Component)]
pub struct Card;

#[derive(Component)]
pub struct CardTitle;

#[derive(Component)]
pub struct CardBody;

#[derive(Component)]
pub struct CardFoot;

#[derive(Component)]
pub struct CardRule;

#[derive(Component)]
pub struct Rising {
    pub age: f32,
    pub alpha: f32,
    pub scale: f32,
}

pub const RISE_HOLD: f32 = 3.0;

fn wrapped() -> TextLayout {
    TextLayout {
        justify: Justify::Center,
        linebreak: LineBreak::WordBoundary,
    }
}

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };

    commands
        .spawn((
            Card,
            UiLayer::Tooltips,
            Pickable::IGNORE,
            GlobalZIndex(1000),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                bottom: px(0.0),
                width: px(WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(5.0),
                padding: UiRect::bottom(px(5.0)),
                ..default()
            },
            ImageNode {
                image: kit.tip_body.clone(),
                image_mode: sliced(6.0),
                ..default()
            },
        ))
        .with_children(|card| {
            card.spawn((
                Pickable::IGNORE,
                Node {
                    width: Val::Percent(100.0),
                    height: px(HEAD),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.tip_head.clone(),
                    image_mode: sliced(6.0),
                    ..default()
                },
            ))
            .with_children(|head| {
                head.spawn((
                    CardTitle,
                    Pickable::IGNORE,
                    Text::new(""),
                    TextFont::from_font_size(NAME_SIZE),
                    TextColor(TITLE),
                    wrapped(),
                ));
            });

            card.spawn((
                CardBody,
                Pickable::IGNORE,
                Node {
                    width: px(TEXT_WIDTH),
                    ..default()
                },
                Text::new(""),
                TextFont::from_font_size(BODY_SIZE),
                TextColor(BODY),
                wrapped(),
            ));

            card.spawn((
                CardRule,
                Pickable::IGNORE,
                Node {
                    width: px(TEXT_WIDTH),
                    height: px(1.0),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            ));

            card.spawn((
                CardFoot,
                Pickable::IGNORE,
                Node {
                    width: px(TEXT_WIDTH),
                    display: Display::None,
                    ..default()
                },
                Text::new(""),
                TextFont::from_font_size(BODY_SIZE),
                TextColor(FOOT),
                wrapped(),
            ));
        });

    commands.spawn((
        Rising {
            age: RISE_HOLD + 10.0,
            alpha: 0.0,
            scale: 1.0,
        },
        Pickable::IGNORE,
        GlobalZIndex(900),
        UiTransform::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            right: px(0.0),
            bottom: px(104.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(""),
        TextFont::from_font_size(9.0),
        TextColor(Color::NONE),
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::NONE,
        },
        wrapped(),
    ));
}

pub fn tooltip(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    wheel: Res<AccumulatedMouseScroll>,
    scroll: Res<Scroll>,
    windows: Query<&Window>,
    hidden: Res<UiHidden>,
    options: Res<Options>,
    mut hover: ResMut<Hover>,
    mut cards: Query<(&mut Node, &mut Visibility, &ComputedNode), With<Card>>,
    mut titles: Query<&mut Text, (With<CardTitle>, Without<CardBody>, Without<CardFoot>)>,
    mut bodies: Query<&mut Text, (With<CardBody>, Without<CardTitle>, Without<CardFoot>)>,
    mut feet: Query<
        (&mut Text, &mut Node, &mut TextColor),
        (
            With<CardFoot>,
            Without<CardTitle>,
            Without<CardBody>,
            Without<Card>,
        ),
    >,
    mut rules: Query<&mut Node, (With<CardRule>, Without<Card>, Without<CardFoot>)>,
) {
    let dt = time.delta_secs();
    hover.wheel_quiet = if wheel.delta != Vec2::ZERO {
        WHEEL_QUIET
    } else {
        (hover.wheel_quiet - dt).max(0.0)
    };

    let Some(window) = windows.iter().next() else {
        return;
    };
    let scale = layer_scale(UiLayer::Tooltips, window_size(window), &options);
    let screen = window_size(window) / scale;
    let cursor = cursor_canvas(window, scale);

    if let Some((waited, from)) = hover.leaving {
        let from = if waited == 0.0 {
            cursor.unwrap_or(from)
        } else {
            from
        };
        let drifted = cursor.is_some_and(|at| at.distance(from) * scale > LEAVE_DRIFT);
        let waited = waited + dt.max(f32::EPSILON);

        if waited >= LEAVE_DELAY || drifted {
            hover.hide();
        } else {
            hover.leaving = Some((waited, from));
        }
    }

    if hover.visible {
        hover.shown_for += dt;
        let interrupted = buttons.get_just_pressed().next().is_some()
            || wheel.delta != Vec2::ZERO
            || scroll.dragged && scroll.held.is_some();

        if interrupted && hover.shown_for >= MIN_SHOWN {
            hover.hide();
        }
    }

    let showing = hover.visible && !hidden.0 && hover.content.is_some() && options.on("tooltips");

    for (mut node, mut visibility, computed) in &mut cards {
        visibility.set_if_neq(if showing {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });

        if !showing {
            continue;
        }

        let Some(at) = cursor else { continue };
        let size = computed.size() * computed.inverse_scale_factor();
        let width = size.x.max(WIDTH);
        let height = size.y;
        let left = (at.x - width * 0.5).clamp(2.0, (screen.x - width - 2.0).max(2.0));
        let above = at.y + MARGIN;
        let bottom = if above + height > screen.y - 2.0 {
            (at.y - MARGIN - height).max(2.0)
        } else {
            above
        };

        node.left = px(left);
        node.bottom = px(bottom);
    }

    if hover.fresh {
        hover.fresh = false;

        if let Some(content) = hover.content.clone() {
            for mut text in &mut titles {
                **text = content.title.clone();
            }

            for mut text in &mut bodies {
                **text = content.body.clone();
            }

            for (mut text, mut node, mut color) in &mut feet {
                node.display = if content.foot.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                **text = content.foot.clone().unwrap_or_default();
                color.0 = if content.warn { WARN } else { FOOT };
            }

            for mut node in &mut rules {
                node.display = if content.foot.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

pub fn notice(
    time: Res<Time>,
    open: Res<OpenWindow>,
    mut asked: MessageReader<Notice>,
    mut rising: Query<(
        &mut Rising,
        &mut Text,
        &mut TextColor,
        &mut TextShadow,
        &mut UiTransform,
    )>,
) {
    let fresh = asked.read().last().cloned();
    let dt = time.delta_secs();

    for (mut rise, mut text, mut color, mut shadow, mut transform) in &mut rising {
        if let Some(Notice(said)) = &fresh {
            **text = said.clone();
            rise.age = 0.0;
            rise.alpha = 0.0;
            rise.scale = 1.5;
        }

        rise.age += dt;

        if open.0.is_some() {
            rise.alpha = 0.0;
        } else if rise.age < RISE_HOLD {
            rise.alpha = (rise.alpha + dt * 3.0).min(1.0);
        } else {
            rise.alpha = (rise.alpha - dt * 2.0).max(0.0);
        }

        rise.scale = (rise.scale - dt * 3.0).max(1.0);
        color.0 = Color::srgba(1.0, 1.0, 1.0, rise.alpha);
        shadow.color = Color::srgba(0.0, 0.0, 0.0, rise.alpha * 0.8);
        transform.scale = Vec2::splat(rise.scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::find;

    #[test]
    fn a_ready_button_shows_its_hint_and_a_planned_one_says_what_is_missing() {
        let wolf = Tip::of(find("wolf").expect("wolf"));
        assert_eq!(
            wolf.foot.as_deref(),
            Some("Hold Shift to drop several at once.")
        );
        assert!(!wolf.warn);

        let fuse = Tip::of(find("fuse").expect("fuse"));
        assert!(fuse.warn);
        assert!(fuse.foot.unwrap().contains("fuses"));

        assert!(!Tip::of(find("tnt").expect("tnt")).warn);
    }

    #[test]
    fn leaving_a_button_hides_the_card_after_a_short_delay() {
        let mut hover = Hover::default();
        hover.enter(Entity::PLACEHOLDER, Tip::plain("a", "b."));
        assert!(hover.visible);

        hover.leave();
        assert!(hover.visible);

        hover.hide();
        assert!(!hover.visible);
    }
}
