//! The controls that slide in from the left: cancel, brush size, world speed, and the way back from a hidden interface.

use crate::art::{Kit, sliced};
use crate::bar::{BarRoot, badge_node, icon_node, px};
use crate::layout::{BUTTON, ICON};
use crate::motion::Mover;
use crate::press::{hover_pop, press_pop};
use crate::size::Preview;
use crate::sound::Cue;
use crate::tips::Tip;
use crate::{Chosen, Controls, OpenWindow, Panel, SpeedRow, UiHidden};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_brush::{Brush, PRESETS};
use econbox_clock::{ON_PANEL, SPEEDS, WorldClock};

pub const SLIDE: Vec2 = Vec2::new(-175.0, 0.0);
pub const SPEED_OFFSETS: [f32; ON_PANEL] = [33.0, 65.8, 98.8, 131.8, 164.8, 197.8];
const STACK_X: f32 = 25.0;
const CANCEL_Y: f32 = 135.0;
const UPPER_Y: f32 = 185.0;
const CANCEL_HEIGHT: f32 = 54.0;
const UNHIDE: f32 = 26.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Cancel,
    Brush,
    Clock,
    Unhide,
}

#[derive(Component)]
pub struct CancelButton;

#[derive(Component)]
pub struct CancelIcon;

#[derive(Component)]
pub struct BrushButton;

#[derive(Component)]
pub struct BrushIcon;

#[derive(Component)]
pub struct ClockButton;

#[derive(Component)]
pub struct SpeedRowNode;

#[derive(Component, Debug, Clone, Copy)]
pub struct SpeedButton(pub usize);

#[derive(Component)]
pub struct UnhideButton;

fn holder(side: Side, centre: Vec2, hidden: Vec2) -> impl Bundle {
    (
        side,
        Mover::new(hidden, false),
        UiTransform::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(centre.x),
            bottom: px(centre.y),
            width: px(0.0),
            height: px(0.0),
            ..default()
        },
        ZIndex(20),
    )
}

fn centred(width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(-width * 0.5),
        bottom: px(-height * 0.5),
        width: px(width),
        height: px(height),
        ..default()
    }
}

fn plain(kit: &Kit) -> ImageNode {
    ImageNode {
        image: kit.button.clone(),
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };

    commands
        .spawn(holder(Side::Cancel, Vec2::new(STACK_X, CANCEL_Y), SLIDE))
        .with_children(|holder| {
            holder
                .spawn((
                    CancelButton,
                    Hovered::default(),
                    Tip::plain("Cancel", "Put the selected power away."),
                    UiTransform::default(),
                    centred(BUTTON, CANCEL_HEIGHT),
                    ImageNode {
                        image: kit.button.clone(),
                        image_mode: sliced(6.0),
                        ..default()
                    },
                ))
                .with_children(|button| {
                    let inset = (BUTTON - ICON) * 0.5;
                    button.spawn((
                        CancelIcon,
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(inset),
                            top: px(CANCEL_HEIGHT * 0.5 - 10.5 - ICON * 0.5),
                            width: px(ICON),
                            height: px(ICON),
                            ..default()
                        },
                        ImageNode::default(),
                    ));
                    button.spawn((
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(inset),
                            top: px(CANCEL_HEIGHT * 0.5 + 15.0 - ICON * 0.5),
                            width: px(ICON),
                            height: px(ICON),
                            ..default()
                        },
                        ImageNode::new(kit.close.clone()),
                    ));
                });
        });

    commands
        .spawn(holder(Side::Brush, Vec2::new(STACK_X, UPPER_Y), SLIDE))
        .with_children(|holder| {
            holder
                .spawn((
                    BrushButton,
                    Hovered::default(),
                    Tip::plain(
                        "Brush Size",
                        "Pick the shape and size your power paints with.",
                    ),
                    UiTransform::default(),
                    centred(BUTTON, BUTTON),
                    ImageNode {
                        image: kit.studded.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        BrushIcon,
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(BUTTON * 0.5),
                            top: px(BUTTON * 0.5),
                            ..default()
                        },
                        ImageNode::default(),
                    ));
                });
        });

    commands
        .spawn(holder(Side::Clock, Vec2::new(STACK_X, UPPER_Y), SLIDE))
        .with_children(|holder| {
            holder
                .spawn((
                    ClockButton,
                    Hovered::default(),
                    Tip::plain("World Speed", "Open the list of speeds."),
                    UiTransform::default(),
                    centred(BUTTON, BUTTON),
                    plain(&kit),
                ))
                .with_children(|button| {
                    button.spawn((
                        Pickable::IGNORE,
                        icon_node(),
                        ImageNode::new(kit.hourglass.clone()),
                    ));
                    button.spawn((
                        crate::press::ClockBadge,
                        Pickable::IGNORE,
                        badge_node(),
                        ImageNode::new(kit.badges[1].clone()),
                    ));
                });

            holder
                .spawn((
                    SpeedRowNode,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        bottom: px(0.0),
                        width: px(0.0),
                        height: px(0.0),
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|row| {
                    for (rung, offset) in SPEED_OFFSETS.iter().enumerate() {
                        let speed = SPEEDS[rung];
                        row.spawn((
                            SpeedButton(rung),
                            Hovered::default(),
                            Tip::plain(speed.label, "Run the world at this speed."),
                            UiTransform::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(offset - BUTTON * 0.5),
                                bottom: px(-BUTTON * 0.5),
                                width: px(BUTTON),
                                height: px(BUTTON),
                                ..default()
                            },
                            plain(&kit),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Pickable::IGNORE,
                                icon_node(),
                                ImageNode::new(kit.hourglass.clone()),
                            ));
                            button.spawn((
                                Pickable::IGNORE,
                                badge_node(),
                                ImageNode::new(kit.badges[rung].clone()),
                            ));
                        });
                    }
                });
        });

    commands
        .spawn(holder(
            Side::Unhide,
            Vec2::new(20.0, 20.0),
            Vec2::new(-40.0, -40.0),
        ))
        .with_children(|holder| {
            holder
                .spawn((
                    UnhideButton,
                    Hovered::default(),
                    Tip::plain("Show Interface", "Bring the panels back."),
                    UiTransform::default(),
                    centred(UNHIDE, UNHIDE),
                    ImageNode {
                        image: kit.button.clone(),
                        image_mode: sliced(6.0),
                        ..default()
                    },
                ))
                .with_children(|button| {
                    let side = 22.0;
                    button.spawn((
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px((UNHIDE - side) * 0.5),
                            top: px((UNHIDE - side) * 0.5),
                            width: px(side),
                            height: px(side),
                            ..default()
                        },
                        ImageNode::new(kit.eye.clone()),
                    ));
                });
        });
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    cancels: Query<(), With<CancelButton>>,
    brushes: Query<(), With<BrushButton>>,
    clocks: Query<Entity, With<ClockButton>>,
    speeds: Query<&SpeedButton>,
    unhides: Query<(), With<UnhideButton>>,
    mut open: ResMut<OpenWindow>,
    mut controls: Controls,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;
    let pressed = cancels.contains(entity)
        || brushes.contains(entity)
        || clocks.contains(entity)
        || speeds.contains(entity)
        || unhides.contains(entity);

    if pressed {
        cues.write(Cue::Click);
    }

    if cancels.get(entity).is_ok() {
        click.propagate(false);
        commands.entity(entity).insert(press_pop());
        controls.deselect();
    } else if brushes.get(entity).is_ok() {
        click.propagate(false);
        commands.entity(entity).insert(press_pop());
        open.0 = Some(Panel::Brushes);
    } else if clocks.get(entity).is_ok() {
        click.propagate(false);
        commands.entity(entity).insert(press_pop());
        controls.speeds.0 = !controls.speeds.0;
    } else if let Ok(speed) = speeds.get(entity) {
        click.propagate(false);
        controls.clock.select(SPEEDS[speed.0].id);
        controls.speeds.0 = false;

        for clock in &clocks {
            commands.entity(clock).insert(press_pop());
        }
    } else if unhides.get(entity).is_ok() {
        click.propagate(false);
        commands.entity(entity).insert(hover_pop());
        controls.hidden.0 = false;
    }
}

pub fn show(
    chosen: Res<Chosen>,
    open: Res<OpenWindow>,
    hidden: Res<UiHidden>,
    brush: Res<Brush>,
    preview: Res<Preview>,
    kit: Option<Res<Kit>>,
    mut speeds: ResMut<SpeedRow>,
    mut clock: ResMut<WorldClock>,
    mut movers: Query<(&Side, &mut Mover)>,
    mut bar: Query<&mut Mover, (With<BarRoot>, Without<Side>)>,
    mut rows: Query<&mut Node, With<SpeedRowNode>>,
    mut cancel_icons: Query<&mut ImageNode, (With<CancelIcon>, Without<BrushIcon>)>,
    mut brush_icons: Query<(&mut ImageNode, &mut Node), (With<BrushIcon>, Without<SpeedRowNode>)>,
) {
    let window = open.0.is_some();
    let def = chosen.def();
    let clock_open = def.is_some_and(|def| def.id == "clock") && !window;

    if clock.hold != window {
        clock.hold = window;
    }

    if !clock_open && speeds.0 {
        speeds.0 = false;
    }

    for (side, mut mover) in &mut movers {
        let visible = match side {
            Side::Cancel => def.is_some() && !window,
            Side::Brush => def.is_some_and(|def| def.sizes) && !window,
            Side::Clock => clock_open,
            Side::Unhide => hidden.0,
        };
        mover.show(visible);
    }

    for mut mover in &mut bar {
        mover.show((!window && !hidden.0) || preview.ghost);
    }

    for mut node in &mut rows {
        let wanted = if speeds.0 {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != wanted {
            node.display = wanted;
        }
    }

    let Some(kit) = kit else { return };

    if let Some(def) = def
        && let Some(image) = kit.icons.get(def.id)
    {
        for mut icon in &mut cancel_icons {
            if icon.image != *image {
                icon.image = image.clone();
            }
        }
    }

    let at = brush.preset().unwrap_or(econbox_brush::DEFAULT_PRESET);
    let (image, size) = &kit.brushes[at.min(PRESETS.len() - 1)];

    for (mut icon, mut node) in &mut brush_icons {
        if icon.image != *image {
            icon.image = image.clone();
        }

        let wanted = (px(size.x), px(size.y), px(-size.x * 0.5), px(-size.y * 0.5));
        if node.width != wanted.0 || node.height != wanted.1 {
            node.width = wanted.0;
            node.height = wanted.1;
            node.margin = UiRect {
                left: wanted.2,
                top: wanted.3,
                ..default()
            };
        }
    }
}
