//! Builds the bar once the plating exists, then hangs an icon on every slot.

use super::chrome::{
    Chrome, ToolArt, action_slot, badge_slot, cap_slice, face, panel_slice, pip, plated,
    post_slice, rail, socket, socket_slice, tab_slice, tool_slot,
};
use super::{
    Action, ActionIcon, ActionSlot, Bar, CLEAR, DIM, GroupBox, SLOT_GAP, STACK, SpeciesIcon,
    Status, TABS, Tab, TabSlot, TabStrip, ToolIcon, ToolSlot,
};
use crate::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub(super) fn spawn(mut commands: Commands, chrome: Option<Res<Chrome>>) {
    let Some(chrome) = chrome else { return };

    commands
        .spawn((
            Bar,
            RelativeCursorPosition::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            },
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    margin: UiRect::new(Val::Px(10.0), Val::Px(0.0), Val::Px(0.0), Val::Px(38.0)),
                    padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                    ..default()
                },
                plated(&chrome.slot, socket_slice()),
            ))
            .with_children(|strip| {
                strip.spawn((
                    Status,
                    Text::new(""),
                    TextFont::from_font_size(11.0),
                    TextColor(DIM),
                ));
            });

            bar.spawn((
                ZIndex(1),
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    padding: UiRect::new(Val::Px(4.0), Val::Px(10.0), Val::Px(9.0), Val::Px(7.0)),
                    ..default()
                },
                plated(&chrome.panel, panel_slice()),
                face(art::FRAME),
            ))
            .with_children(|panel| {
                panel
                    .spawn((
                        TabStrip,
                        RelativeCursorPosition::default(),
                        ZIndex(2),
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(-32.0),
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            height: Val::Px(36.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(2.0),
                            ..default()
                        },
                    ))
                    .with_children(|tabs| {
                        for tab in TABS {
                            tabs.spawn((
                                TabSlot(tab),
                                Button,
                                ActivateOnPress,
                                PointerHovered::default(),
                                Node {
                                    width: Val::Px(56.0),
                                    height: Val::Px(36.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::bottom(Val::Px(4.0)),
                                    ..default()
                                },
                                plated(&chrome.tab, tab_slice()),
                            ))
                            .with_children(|tab_slot| {
                                tab_slot.spawn((
                                    TabSlot(tab),
                                    Node {
                                        width: Val::Px(26.0),
                                        height: Val::Px(26.0),
                                        ..default()
                                    },
                                ));
                            });
                        }
                    });

                panel.spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(STACK + 6.0),
                        margin: UiRect::right(Val::Px(4.0)),
                        ..default()
                    },
                    plated(&chrome.cap, cap_slice()),
                    face(art::FRAME),
                ));
                panel
                    .spawn((
                        ActionSlot(Action::Undo),
                        Button,
                        ActivateOnPress,
                        PointerHovered::default(),
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(STACK),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        plated(&chrome.button, post_slice()),
                        face(art::EMBER),
                        BorderColor::all(CLEAR),
                    ))
                    .with_children(|slot| {
                        slot.spawn((ActionIcon(Action::Undo), pip()));
                    });

                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(SLOT_GAP),
                        ..default()
                    })
                    .with_children(|cluster| {
                        for action in [Action::Play, Action::Speed] {
                            cluster
                                .spawn((ActionSlot(action), socket(&chrome, Display::Flex)))
                                .with_children(|slot| {
                                    slot.spawn((ActionIcon(action), pip()));
                                });
                        }
                    });

                panel.spawn(rail(&chrome, Display::Flex));

                for tab in TABS {
                    let shown = if tab == Tab::default() {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    let groups = tab.groups();
                    let last = groups.len().saturating_sub(1);

                    for (at, group) in groups.into_iter().enumerate() {
                        panel
                            .spawn((
                                GroupBox(tab),
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    flex_wrap: FlexWrap::Wrap,
                                    height: Val::Px(STACK),
                                    column_gap: Val::Px(SLOT_GAP),
                                    row_gap: Val::Px(SLOT_GAP),
                                    margin: UiRect::right(Val::Px(4.0)),
                                    display: shown,
                                    ..default()
                                },
                            ))
                            .with_children(|grid| {
                                for tool in group {
                                    grid.spawn((ToolSlot(tool), socket(&chrome, Display::Flex)))
                                        .with_children(|slot| match tool {
                                            Tool::Spawn(species) => {
                                                slot.spawn((SpeciesIcon(species), pip()));
                                            }
                                            tool => {
                                                slot.spawn((ToolIcon(tool), pip()));
                                            }
                                        });
                                }
                            });

                        if at < last {
                            panel.spawn((GroupBox(tab), rail(&chrome, shown)));
                        }
                    }
                }

                panel.spawn(rail(&chrome, Display::Flex));

                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_wrap: FlexWrap::Wrap,
                        height: Val::Px(STACK),
                        column_gap: Val::Px(SLOT_GAP),
                        row_gap: Val::Px(SLOT_GAP),
                        ..default()
                    })
                    .with_children(|cluster| {
                        for action in [Action::Shape, Action::Flood, Action::Shrink, Action::Grow] {
                            cluster
                                .spawn((ActionSlot(action), socket(&chrome, Display::Flex)))
                                .with_children(|slot| {
                                    slot.spawn((ActionIcon(action), pip()));
                                });
                        }
                    });
            });
        });
}

pub(super) fn dress(
    mut commands: Commands,
    beasts: Option<Res<CreatureArt>>,
    tools: Option<Res<ToolArt>>,
    species: Query<(Entity, &SpeciesIcon), Without<ImageNode>>,
    kit: Query<(Entity, &ToolIcon), Without<ImageNode>>,
    actions: Query<(Entity, &ActionIcon), Without<ImageNode>>,
    badges: Query<(Entity, &TabSlot), (Without<ImageNode>, Without<Button>)>,
) {
    if let Some(art) = beasts {
        for (entity, icon) in &species {
            commands.entity(entity).insert(ImageNode::from_atlas_image(
                art.image.clone(),
                TextureAtlas {
                    layout: art.layout.clone(),
                    index: icon.0.slot(),
                },
            ));
        }
    }

    let Some(art) = tools else { return };
    let mut paint = |entity: Entity, index: usize| {
        commands.entity(entity).insert(ImageNode::from_atlas_image(
            art.image.clone(),
            TextureAtlas {
                layout: art.layout.clone(),
                index,
            },
        ));
    };

    for (entity, icon) in &kit {
        if let Some(index) = tool_slot(icon.0) {
            paint(entity, index);
        }
    }

    for (entity, icon) in &actions {
        paint(entity, action_slot(icon.0));
    }

    for (entity, badge) in &badges {
        paint(entity, badge_slot(badge.0));
    }
}
