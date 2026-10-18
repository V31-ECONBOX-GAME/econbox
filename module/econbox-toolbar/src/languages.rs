//! The languages window: a grid of the languages the game can speak, and the call for translators.

use crate::Panel;
use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::press::press_pop;
use crate::sound::Cue;
use crate::tips::Tip;
use crate::windows::{CONTENT_WIDTH, shadow, spawn_frame, spawn_view};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_art::Glyph;
use econbox_art::window::Scene;

pub const CELL: Vec2 = Vec2::new(62.0, 20.0);
pub const GAP: f32 = 6.0;
pub const COLUMNS: u16 = 3;

pub const LANGUAGES: [(&str, &str); 1] = [("en", "English")];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language(pub &'static str);

impl Default for Language {
    fn default() -> Self {
        Self(LANGUAGES[0].0)
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LanguageButton(pub &'static str);

#[derive(Component)]
pub struct HelpTranslate;

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };
    let (_, background) = spawn_frame(
        &mut commands,
        &kit,
        Panel::Languages,
        kit.sheep.clone(),
        Vec2::new(14.0, 320.0 - 315.2),
    );
    let content = spawn_view(
        &mut commands,
        &kit,
        background,
        Panel::Languages,
        5.0,
        (5.0, 5.0),
        true,
    );

    commands.entity(content).with_children(|list| {
        illustration(list, &kit, Scene::Tide, 169.4915, 208.8);
        grid(list, &kit);
        help(list, &kit);
        illustration(list, &kit, Scene::Dusk, 145.0, 175.0);
    });
}

fn illustration(list: &mut ChildSpawnerCommands, kit: &Kit, scene: Scene, height: f32, mask: f32) {
    let art = kit.scenes.get(&scene).cloned().unwrap_or_default();
    list.spawn((
        Pickable::IGNORE,
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
    .with_children(|frame| {
        frame.spawn((
            Pickable::IGNORE,
            Node {
                width: px(259.0 * 0.78),
                height: px(mask * 0.78),
                ..default()
            },
            ImageNode {
                image: art,
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ));
    });
}

fn grid(list: &mut ChildSpawnerCommands, kit: &Kit) {
    let icon = kit
        .white_glyphs
        .get(&Glyph::Bubbles)
        .cloned()
        .unwrap_or_default();

    list.spawn((
        Node {
            width: px(CONTENT_WIDTH),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(COLUMNS, CELL.x),
            justify_content: JustifyContent::Center,
            column_gap: px(GAP),
            row_gap: px(GAP),
            padding: UiRect::vertical(px(5.0)),
            flex_shrink: 0.0,
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ))
    .with_children(|grid| {
        for (id, name) in LANGUAGES {
            grid.spawn((
                LanguageButton(id),
                Hovered::default(),
                Tip::plain(name, "Use this language for the interface."),
                UiTransform::default(),
                Node {
                    width: px(CELL.x),
                    height: px(CELL.y),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.red_bar.clone(),
                    image_mode: sliced(5.0),
                    ..default()
                },
            ))
            .with_children(|button| {
                button.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(1.0),
                        width: px(28.0 * 0.7),
                        height: px(28.0 * 0.7),
                        ..default()
                    },
                    ImageNode::new(icon.clone()),
                ));
                button
                    .spawn((
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(CELL.x * 0.5 + 7.187 - (CELL.x - 22.374) * 0.5),
                            width: px(CELL.x - 22.374),
                            height: px(15.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|text| {
                        text.spawn((
                            Pickable::IGNORE,
                            Text::new(name),
                            kit.text(7.0),
                            TextColor(Color::WHITE),
                            shadow(),
                        ));
                    });
            });
        }
    });
}

fn help(list: &mut ChildSpawnerCommands, kit: &Kit) {
    let icon = kit.glyphs.get(&Glyph::Globe).cloned().unwrap_or_default();
    let mut tip = Tip::plain("Help Translating", "Bring econbox to your own language.");
    tip.foot = Some("Not available yet: needs translations.".to_string());
    tip.warn = true;

    list.spawn((
        Node {
            width: px(CONTENT_WIDTH),
            height: px(30.0),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            padding: UiRect::vertical(px(5.0)),
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ))
    .with_children(|row| {
        row.spawn((
            HelpTranslate,
            Hovered::default(),
            tip,
            UiTransform::default(),
            Node {
                width: px(135.0),
                height: px(20.0),
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                image: kit.gray_bar.clone(),
                image_mode: sliced(4.0),
                ..default()
            },
        ))
        .with_children(|button| {
            button.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(11.5 - 10.0),
                    width: px(20.0),
                    height: px(20.0),
                    ..default()
                },
                ImageNode::new(icon),
            ));
            button
                .spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(135.0 * 0.5 + 9.4904 - (135.0 - 25.0791) * 0.5),
                        width: px(135.0 - 25.0791),
                        height: px(16.2),
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
                .with_children(|inside| {
                    inside.spawn((
                        Pickable::IGNORE,
                        Text::new("Help translating"),
                        kit.text(7.0),
                        TextColor(Color::WHITE),
                        shadow(),
                    ));
                });
        });
    });
}

pub fn refresh(
    language: Res<Language>,
    kit: Option<Res<Kit>>,
    mut buttons: Query<(&LanguageButton, &mut ImageNode)>,
) {
    if !language.is_changed() {
        return;
    }
    let Some(kit) = kit else { return };

    for (button, mut image) in &mut buttons {
        image.image = if button.0 == language.0 {
            kit.red_bar.clone()
        } else {
            kit.gray_bar.clone()
        };
    }
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    buttons: Query<&LanguageButton>,
    helps: Query<(), With<HelpTranslate>>,
    mut language: ResMut<Language>,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if let Ok(button) = buttons.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        if language.0 != button.0 {
            language.0 = button.0;
        }
    } else if helps.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_the_language_until_another_is_chosen() {
        assert_eq!(Language::default().0, "en");
        assert!(LANGUAGES.iter().any(|(id, _)| *id == "en"));
    }
}
