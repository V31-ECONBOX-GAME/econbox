//! The brush window: every brush shape and size grouped the way the bar offers them.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::layout::BUTTON;
use crate::motion::{Curve, Pop};
use crate::press::press_pop;
use crate::tips::{TITLE, Tip, WINDOW_RATIO};
use crate::{OpenWindow, Panel};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_brush::{Brush, GROUPS, Group, PRESETS};

const WIDTH: f32 = 240.0 * WINDOW_RATIO;
const HEIGHT: f32 = 320.0 * WINDOW_RATIO;
const LIST_WIDTH: f32 = 192.0 * WINDOW_RATIO;
const LIST_TOP: f32 = 34.5 * WINDOW_RATIO;
const CELL: f32 = BUTTON * WINDOW_RATIO;
const SPACING: f32 = 5.0 * WINDOW_RATIO;
const GROUP_GAP: f32 = 10.0 * WINDOW_RATIO;
const PAD: f32 = 3.0 * WINDOW_RATIO;

#[derive(Component)]
pub struct Shade;

#[derive(Component)]
pub struct BrushWindow;

#[derive(Component)]
pub struct CloseWindow;

#[derive(Component, Debug, Clone, Copy)]
pub struct PresetButton(pub usize);

fn tip_for(at: usize) -> Tip {
    let preset = PRESETS[at];

    if preset.flood {
        return Tip::plain(
            "Fill",
            "Paint the whole patch of matching ground under the pointer.",
        );
    }

    Tip::plain(preset.group.name(), "Paint with this brush.")
}

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };

    commands.spawn((
        Shade,
        GlobalZIndex(500),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.0),
            right: px(0.0),
            top: px(0.0),
            bottom: px(0.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
    ));

    commands
        .spawn((
            BrushWindow,
            GlobalZIndex(600),
            UiTransform::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: px(-WIDTH * 0.5),
                    top: px(-HEIGHT * 0.5),
                    ..default()
                },
                width: px(WIDTH),
                height: px(HEIGHT),
                display: Display::None,
                ..default()
            },
            ImageNode {
                image: kit.window.clone(),
                image_mode: sliced(12.0),
                ..default()
            },
        ))
        .with_children(|window| {
            window.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    right: px(0.0),
                    top: px(6.6 * WINDOW_RATIO),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Text::new("Brush"),
                TextFont::from_font_size(10.0 * WINDOW_RATIO),
                TextColor(TITLE),
            ));

            window
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(WIDTH * 0.5 + 104.2 * WINDOW_RATIO - 20.0 * WINDOW_RATIO),
                        top: px(HEIGHT * 0.5 - 152.5 * WINDOW_RATIO - 18.0 * WINDOW_RATIO),
                        width: px(40.0 * WINDOW_RATIO),
                        height: px(36.0 * WINDOW_RATIO),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: kit.close_tab.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    },
                ))
                .with_children(|tab| {
                    tab.spawn((
                        CloseWindow,
                        Hovered::default(),
                        Tip::plain("Close", "Close this window."),
                        UiTransform::default(),
                        Node {
                            width: px(CELL),
                            height: px(CELL),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|close| {
                        close.spawn((
                            Pickable::IGNORE,
                            Node {
                                width: px(24.0 * WINDOW_RATIO),
                                height: px(24.0 * WINDOW_RATIO),
                                ..default()
                            },
                            ImageNode::new(kit.close.clone()),
                        ));
                    });
                });

            window
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    left: px((WIDTH - LIST_WIDTH) * 0.5),
                    top: px(LIST_TOP),
                    width: px(LIST_WIDTH),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(GROUP_GAP),
                    ..default()
                })
                .with_children(|list| {
                    for group in GROUPS {
                        spawn_group(list, &kit, group);
                    }
                });
        });
}

fn spawn_group(list: &mut ChildSpawnerCommands, kit: &Kit, group: Group) {
    list.spawn((
        Node {
            width: px(LIST_WIDTH),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: px(SPACING),
            row_gap: px(SPACING),
            padding: UiRect::all(px(PAD)),
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ))
    .with_children(|grid| {
        for (at, preset) in PRESETS.iter().enumerate() {
            if preset.group != group {
                continue;
            }

            let (image, size) = &kit.brushes[at];
            let size = *size * WINDOW_RATIO;

            grid.spawn((
                PresetButton(at),
                Hovered::default(),
                tip_for(at),
                UiTransform::default(),
                Node {
                    width: px(CELL),
                    height: px(CELL),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.button.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
            ))
            .with_children(|button| {
                button.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(size.x),
                        height: px(size.y),
                        ..default()
                    },
                    ImageNode::new(image.clone()),
                ));
            });
        }
    });
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    presets: Query<&PresetButton>,
    closes: Query<(), With<CloseWindow>>,
    shades: Query<(), With<Shade>>,
    mut brush: ResMut<Brush>,
    mut open: ResMut<OpenWindow>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if let Ok(pick) = presets.get(entity) {
        click.propagate(false);
        commands.entity(entity).insert(press_pop());
        brush.wear(PRESETS[pick.0]);
        open.0 = None;
    } else if closes.get(entity).is_ok() || shades.get(entity).is_ok() {
        click.propagate(false);
        open.0 = None;
    }
}

pub fn show(
    mut commands: Commands,
    open: Res<OpenWindow>,
    mut windows: Query<(Entity, &mut Node), With<BrushWindow>>,
    mut shades: Query<&mut Node, (With<Shade>, Without<BrushWindow>)>,
) {
    if !open.is_changed() {
        return;
    }

    let visible = open.0 == Some(Panel::Brushes);
    let display = if visible {
        Display::Flex
    } else {
        Display::None
    };

    for (entity, mut node) in &mut windows {
        node.display = display;

        if visible {
            commands
                .entity(entity)
                .insert(Pop::new(Vec2::splat(0.9), 0.15, Curve::OutBack));
        }
    }

    for mut node in &mut shades {
        node.display = display;
    }
}
