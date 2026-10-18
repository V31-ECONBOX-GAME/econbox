//! The window behind the gear button: a banner, the community links, and the doors to settings and languages.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::press::press_pop;
use crate::sound::Cue;
use crate::tips::{TITLE, Tip};
use crate::windows::{CONTENT_WIDTH, label, shadow, spawn_frame, spawn_view};
use crate::{OpenWindow, Panel};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_art::Glyph;
use econbox_art::window::Scene;

pub const SCALE: f32 = 1.0749;
pub const LINK: f32 = 27.0 * SCALE;
pub const DOOR: f32 = 32.0 * SCALE;
pub const LINK_LABEL: Color = Color::srgb(0.984, 0.6, 0.11);

pub const LINKS: [(f32, Glyph, &str); 4] = [
    (-74.5, Glyph::Bubbles, "Chat"),
    (-25.4, Glyph::Bell, "News"),
    (24.8, Glyph::Banner, "Forum"),
    (74.5, Glyph::Globe, "Website"),
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Door {
    Settings,
    Languages,
}

#[derive(Component)]
pub struct Link;

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };
    let (_, background) = spawn_frame(
        &mut commands,
        &kit,
        Panel::Other,
        kit.dragon.clone(),
        Vec2::new(17.055, 2.9),
    );
    let content = spawn_view(
        &mut commands,
        &kit,
        background,
        Panel::Other,
        5.0,
        (5.0, 5.0),
        false,
    );

    commands.entity(content).with_children(|list| {
        banner(list, &kit);
        links(list, &kit);
        doors(list, &kit);
    });
}

fn panel(
    list: &mut ChildSpawnerCommands,
    kit: &Kit,
    height: f32,
    fill: impl FnOnce(&mut ChildSpawnerCommands),
) {
    list.spawn((
        Node {
            width: px(CONTENT_WIDTH),
            height: px(height),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ))
    .with_children(fill);
}

fn banner(list: &mut ChildSpawnerCommands, kit: &Kit) {
    let art = kit.scenes.get(&Scene::Void).cloned().unwrap_or_default();
    panel(list, kit, 91.9194, |frame| {
        frame
            .spawn((
                Pickable::IGNORE,
                Node {
                    width: px(259.0 * 0.78),
                    height: px(106.2 * 0.78),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode {
                    image: art,
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
            ))
            .with_children(|art| {
                art.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(2.0),
                        ..default()
                    },
                    Text::new(format!("version: {}", env!("CARGO_PKG_VERSION"))),
                    kit.text(7.0),
                    TextColor(TITLE),
                    shadow(),
                ));
            });
    });
}

fn links(list: &mut ChildSpawnerCommands, kit: &Kit) {
    panel(list, kit, 58.361, |row| {
        for (x, kind, label_text) in LINKS {
            let icon = if kind == Glyph::Globe {
                kit.glyphs.get(&kind)
            } else {
                kit.white_glyphs.get(&kind)
            }
            .cloned()
            .unwrap_or_default();
            let mut tip = Tip::plain(label_text, "A place where players of econbox meet.");
            tip.foot = Some("Not available yet: needs community links.".to_string());
            tip.warn = true;

            row.spawn((
                Link,
                Hovered::default(),
                tip,
                UiTransform::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(CONTENT_WIDTH * 0.5 + x - LINK * 0.5),
                    top: px(19.006 - LINK * 0.5),
                    width: px(LINK),
                    height: px(LINK),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.red_bar.clone(),
                    image_mode: sliced(5.0),
                    ..default()
                },
            ))
            .with_children(|link| {
                link.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(28.0 * SCALE * 0.85),
                        height: px(28.0 * SCALE * 0.85),
                        ..default()
                    },
                    ImageNode::new(icon),
                ));
            });

            label(
                row,
                kit,
                label_text,
                8.0,
                LINK_LABEL,
                Vec2::new(CONTENT_WIDTH * 0.5 + x, 19.006 - LINK * 0.5 + 38.0 * SCALE),
                Vec2::new(46.0 * SCALE, 16.0),
            );
        }
    });
}

fn doors(list: &mut ChildSpawnerCommands, kit: &Kit) {
    panel(list, kit, 115.7404, |row| {
        for (door, x, kind, label_text) in [
            (Door::Settings, -60.0, Glyph::Gear, "Settings"),
            (Door::Languages, 60.0, Glyph::Bubbles, "Languages"),
        ] {
            let icon = kit.glyphs.get(&kind).cloned().unwrap_or_default();
            let tip = match door {
                Door::Settings => {
                    Tip::plain("Settings", "Change how the game looks, sounds and runs.")
                }
                Door::Languages => Tip::plain("Languages", "Pick the language of the interface."),
            };

            row.spawn((
                door,
                Hovered::default(),
                tip,
                UiTransform::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(CONTENT_WIDTH * 0.5 + x - DOOR * 0.5),
                    top: px(24.1 - DOOR * 0.5),
                    width: px(DOOR),
                    height: px(DOOR),
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
                        width: px(28.0 * 1.0749),
                        height: px(28.0 * 1.0749),
                        ..default()
                    },
                    ImageNode::new(icon),
                ));
            });

            label(
                row,
                kit,
                label_text,
                8.0,
                LINK_LABEL,
                Vec2::new(CONTENT_WIDTH * 0.5 + x, 24.1 - DOOR * 0.5 + 40.4 * SCALE),
                Vec2::new(50.0 * SCALE, 18.0),
            );
        }
    });
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    doors: Query<&Door>,
    links: Query<(), With<Link>>,
    mut open: ResMut<OpenWindow>,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if let Ok(door) = doors.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        open.0 = Some(match door {
            Door::Settings => Panel::Settings,
            Door::Languages => Panel::Languages,
        });
    } else if links.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
    }
}
