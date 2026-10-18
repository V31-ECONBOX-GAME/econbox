//! The brush window: every brush shape and size grouped the way the bar offers them.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::layout::BUTTON;
use crate::press::press_pop;
use crate::size::UiLayer;
use crate::sound::Cue;
use crate::tips::{TITLE, Tip};
use crate::windows::{Slide, WindowRoot, frame_art, label};
use crate::{OpenWindow, Panel};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_brush::{Brush, GROUPS, Group, PRESETS};

const WIDTH: f32 = 240.0;
const HEIGHT: f32 = 320.0;
const LIST_WIDTH: f32 = 192.0;
const LIST_TOP: f32 = 34.5;
const CELL: f32 = BUTTON;
const SPACING: f32 = 5.0;
const GROUP_GAP: f32 = 10.0;
const PAD: f32 = 3.0;

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

    commands
        .spawn((
            BrushWindow,
            WindowRoot {
                panel: Panel::Brushes,
                width: WIDTH,
            },
            Slide::default(),
            UiLayer::Windows,
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
        ))
        .with_children(|window| {
            frame_art(window, &kit);
            label(
                window,
                &kit,
                "Brush",
                10.0,
                TITLE,
                Vec2::new(WIDTH * 0.5, HEIGHT * 0.5 - 146.4),
                Vec2::new(130.0, 14.0),
            );

            window
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(WIDTH * 0.5 + 104.2 - 20.0),
                        top: px(HEIGHT * 0.5 - 152.5 - 18.0),
                        width: px(40.0),
                        height: px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: kit.turret.clone(),
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
                                width: px(24.0),
                                height: px(24.0),
                                ..default()
                            },
                            ImageNode::new(kit.cross.clone()),
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
    mut brush: ResMut<Brush>,
    mut open: ResMut<OpenWindow>,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if let Ok(pick) = presets.get(entity) {
        click.propagate(false);
        commands.entity(entity).insert(press_pop());
        cues.write(Cue::Click);
        brush.wear(PRESETS[pick.0]);
        open.0 = None;
    } else if closes.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        open.0 = None;
    }
}
