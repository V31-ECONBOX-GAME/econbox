//! The two New World windows: picking a template, then shaping it before a fresh world is made.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::options::Mark;
use crate::press::press_pop;
use crate::sound::Cue;
use crate::tips::Tip;
use crate::windows::{
    CONTENT_WIDTH, Exit, FRAME_WIDTH, Toward, VIEW_HEIGHT, VIEW_TOP, label, shadow, spawn_frame,
    spawn_view_with,
};
use crate::{OpenWindow, Panel};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_art::window::Scene;
use econbox_art::{Glyph, Icon};
use econbox_worldgen::{
    MAP_SIZES, MapGenSetting, MapGenValues, MapSize, TEMPLATE_IDS, TemplateId, WorldConfig, setting,
};
use std::collections::HashMap;

pub const CELL: f32 = 40.0;
pub const CELL_GAP: Vec2 = Vec2::new(14.0, 4.26);
pub const COLUMNS: u16 = 4;
pub const ROW: Vec2 = Vec2::new(215.0, 25.0);
pub const SWITCH: Vec2 = Vec2::new(46.21, 21.0);
pub const STEP_BUTTON: f32 = 21.0;
pub const FOOTER: f32 = 31.0;
pub const PREVIEW_SIDE: u32 = 64;
pub const PREVIEW_WORLD: u32 = 256;
pub const DISABLED: Color = Color::srgb(0.8, 0.8, 0.8);

pub const ROW_ORDER: [&str; 18] = [
    "gen_random_shapes",
    "gen_cubicles_sizes",
    "gen_perlin_scale_stage_1",
    "gen_perlin_scale_stage_2",
    "gen_perlin_scale_stage_3",
    "gen_random_biomes",
    "gen_mountain_edges",
    "gen_add_vegetation",
    "gen_add_resources",
    "gen_add_center_lake",
    "gen_add_center_land",
    "gen_round_edges",
    "gen_square_edges",
    "gen_ring_effect",
    "gen_low_ground",
    "gen_high_ground",
    "gen_remove_mountains",
    "gen_forbidden_knowledge",
];

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateWorld(pub WorldConfig);

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct WorldDraft {
    pub config: WorldConfig,
    edited: HashMap<TemplateId, MapGenValues>,
}

impl WorldDraft {
    pub fn pick(&mut self, template: TemplateId) {
        self.edited.insert(self.config.template, self.config.values);
        let values = self
            .edited
            .get(&template)
            .copied()
            .unwrap_or(template.template().values);
        self.config.template = template;
        self.config.values = values;
    }

    pub fn reset(&mut self) {
        self.config.reset_values();
        self.edited.remove(&self.config.template);
    }

    pub fn grow(&mut self) {
        self.config.size = step_size(self.config.size, 1);
    }

    pub fn shrink(&mut self) {
        self.config.size = step_size(self.config.size, -1);
    }

    pub fn shown(&self, setting: &MapGenSetting) -> bool {
        setting.is_allowed(self.config.template.template())
    }
}

pub fn step_size(size: MapSize, by: i32) -> MapSize {
    let at = MAP_SIZES.iter().position(|s| *s == size).unwrap_or(0) as i32;
    let count = MAP_SIZES.len() as i32;
    MAP_SIZES[(at + by).rem_euclid(count) as usize]
}

pub fn template_name(id: TemplateId) -> &'static str {
    match id {
        TemplateId::Continent => "Continent",
        TemplateId::BoxWorld => "Box World",
        TemplateId::Islands => "Islands",
        TemplateId::Toast => "Toast",
        TemplateId::Pancake => "Pancake",
        TemplateId::BoringPlains => "Boring Plains",
        TemplateId::Checkerboard => "Checkerboard",
        TemplateId::Cubicles => "Cubicles",
        TemplateId::DormantVolcano => "Dormant Volcano",
        TemplateId::Cheese => "Cheese",
        TemplateId::BadApple => "Bad Apple",
        TemplateId::Donut => "Donut",
        TemplateId::Lasagna => "Lasagna",
        TemplateId::ChaosPearl => "Chaos Pearl",
        TemplateId::Anthill => "Anthill",
        TemplateId::Empty => "Empty",
    }
}

pub fn template_about(id: TemplateId) -> &'static str {
    match id {
        TemplateId::Continent => "One big land in the middle of the sea, with snowy peaks.",
        TemplateId::BoxWorld => "Land fenced in by mountains along every edge.",
        TemplateId::Islands => "Many islands scattered across open water.",
        TemplateId::Toast => "A square slab of land with soft edges and no mountains.",
        TemplateId::Pancake => "A flat round land without mountains.",
        TemplateId::BoringPlains => "Wide plains with barely any water.",
        TemplateId::Checkerboard => "Land and water laid out in squares.",
        TemplateId::Cubicles => "Rooms of land walled off by mountains.",
        TemplateId::DormantVolcano => "A raised island around a quiet peak.",
        TemplateId::Cheese => "Land full of holes and pools.",
        TemplateId::BadApple => "A round land with a mountain crater at its heart.",
        TemplateId::Donut => "A ring of land around a central lake.",
        TemplateId::Lasagna => "Low layered land in a square sea.",
        TemplateId::ChaosPearl => "A low round land shaped by noise.",
        TemplateId::Anthill => "Tunnels carved through solid rock.",
        TemplateId::Empty => "Nothing but ocean, ready to be shaped.",
    }
}

pub fn setting_name(id: &str) -> &'static str {
    match id {
        "gen_random_shapes" => "Random Shapes",
        "gen_cubicles_sizes" => "Cubicle Size",
        "gen_perlin_scale_stage_1" => "Main Noise",
        "gen_perlin_scale_stage_2" => "Detail Noise",
        "gen_perlin_scale_stage_3" => "Bonus Noise",
        "gen_random_biomes" => "Random Biomes",
        "gen_mountain_edges" => "Mountain Edges",
        "gen_add_vegetation" => "Vegetation",
        "gen_add_resources" => "Resources",
        "gen_add_center_lake" => "Center Lake",
        "gen_add_center_land" => "Center Land",
        "gen_round_edges" => "Round Edges",
        "gen_square_edges" => "Square Edges",
        "gen_ring_effect" => "Ring Effect",
        "gen_low_ground" => "Low Ground",
        "gen_high_ground" => "High Ground",
        "gen_remove_mountains" => "Remove Mountains",
        "gen_forbidden_knowledge" => "Forbidden Knowledge",
        _ => "Setting",
    }
}

pub fn setting_mark(id: &str) -> Mark {
    match id {
        "gen_random_shapes" => Mark::Glyph(Glyph::Swirl),
        "gen_cubicles_sizes" => Mark::Icon(Icon::Block),
        "gen_perlin_scale_stage_1" => Mark::Icon(Icon::Wave),
        "gen_perlin_scale_stage_2" => Mark::Icon(Icon::Cloud),
        "gen_perlin_scale_stage_3" => Mark::Icon(Icon::Snowflake),
        "gen_random_biomes" => Mark::Icon(Icon::Flower),
        "gen_mountain_edges" => Mark::Icon(Icon::Peaks),
        "gen_add_vegetation" => Mark::Icon(Icon::Sprout),
        "gen_add_resources" => Mark::Glyph(Glyph::Gem),
        "gen_add_center_lake" => Mark::Icon(Icon::Droplet),
        "gen_add_center_land" => Mark::Icon(Icon::Mound),
        "gen_round_edges" => Mark::Icon(Icon::Round),
        "gen_square_edges" => Mark::Icon(Icon::Square),
        "gen_ring_effect" => Mark::Glyph(Glyph::Rings),
        "gen_low_ground" => Mark::Icon(Icon::Lower),
        "gen_high_ground" => Mark::Icon(Icon::Raise),
        "gen_remove_mountains" => Mark::Icon(Icon::Level),
        _ => Mark::Glyph(Glyph::Brain),
    }
}

pub fn preview_pixels(template: TemplateId) -> Vec<[u8; 4]> {
    let map = econbox_worldgen::Generator::new(
        PREVIEW_WORLD,
        0x0005_eed5,
        template.template(),
        template.template().values,
    )
    .run_all()
    .into_map();

    let step = PREVIEW_WORLD / PREVIEW_SIDE;
    (0..PREVIEW_SIDE)
        .flat_map(|y| (0..PREVIEW_SIDE).map(move |x| (x, y)))
        .map(|(x, y)| {
            let at = ((y * step + step / 2) * PREVIEW_WORLD + x * step + step / 2) as usize;
            let [r, g, b] = map.tiles[at].color();
            [r, g, b, 255]
        })
        .collect()
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TemplateButton(pub TemplateId);

#[derive(Component, Debug, Clone, Copy)]
pub struct SettingRow(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct StepButton(pub &'static str, pub bool);

#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchButton(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchMark(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchLabel(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct Counter(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct RowInk(pub &'static str);

#[derive(Component)]
pub struct SizeButton;

#[derive(Component)]
pub struct SizeLabel;

#[derive(Component)]
pub struct SizeWarning;

#[derive(Component)]
pub struct Blurb;

#[derive(Component)]
pub struct Preview;

#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct CreateButton;

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };
    spawn_templates(&mut commands, &kit);
    spawn_custom(&mut commands, &kit);
}

fn panel(
    list: &mut ChildSpawnerCommands,
    kit: &Kit,
    size: Vec2,
    fill: impl FnOnce(&mut ChildSpawnerCommands),
) {
    list.spawn((
        Node {
            width: px(size.x),
            height: px(size.y),
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

fn illustration(list: &mut ChildSpawnerCommands, kit: &Kit, scene: Scene, size: Vec2, mask: f32) {
    let art = kit.scenes.get(&scene).cloned().unwrap_or_default();
    panel(list, kit, size, |frame| {
        frame.spawn((
            Pickable::IGNORE,
            Node {
                width: px((size.x - 8.0).min(259.0 * 0.78)),
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

fn spawn_templates(commands: &mut Commands, kit: &Kit) {
    let (_, background) = spawn_frame(
        commands,
        kit,
        Panel::NewWorld,
        kit.dragon.clone(),
        Vec2::new(16.9, 320.0 - 315.7),
    );
    let content = spawn_view_with(
        commands,
        kit,
        background,
        Panel::NewWorld,
        5.0,
        (5.0, 5.0),
        true,
        VIEW_HEIGHT,
    );

    commands.entity(content).with_children(|list| {
        illustration(
            list,
            kit,
            Scene::Grove,
            Vec2::new(CONTENT_WIDTH, 145.0),
            175.4179,
        );

        panel(list, kit, Vec2::new(CONTENT_WIDTH, 36.849), |row| {
            let icon = kit.glyphs.get(&Glyph::Globe).cloned().unwrap_or_default();
            row.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(CONTENT_WIDTH * 0.5 - 81.7 - 10.86),
                    width: px(21.7195),
                    height: px(21.7195),
                    ..default()
                },
                ImageNode::new(icon),
            ));
            label(
                row,
                kit,
                "Pick a template to start a new world.",
                8.0,
                Color::WHITE,
                Vec2::new(CONTENT_WIDTH * 0.5 + 9.9375, 36.849 * 0.5),
                Vec2::new(155.005, 29.7116),
            );
        });

        panel(list, kit, Vec2::new(CONTENT_WIDTH, 182.8938), |grid| {
            grid.spawn(Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(COLUMNS, CELL),
                column_gap: px(CELL_GAP.x),
                row_gap: px(CELL_GAP.y),
                ..default()
            })
            .with_children(|cells| {
                for id in TEMPLATE_IDS {
                    let preview = kit.template_previews.get(&id).cloned().unwrap_or_default();
                    cells
                        .spawn((
                            TemplateButton(id),
                            Hovered::default(),
                            Tip::plain(template_name(id), template_about(id)),
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
                                    width: px(CELL - 8.0),
                                    height: px(CELL - 8.0),
                                    ..default()
                                },
                                ImageNode::new(preview),
                            ));
                        });
                }
            });
        });
    });
}

fn spawn_custom(commands: &mut Commands, kit: &Kit) {
    let (_, background) = spawn_frame(
        commands,
        kit,
        Panel::NewWorldCustom,
        kit.crab.clone(),
        Vec2::new(16.9, 320.0 - 315.7),
    );
    let content = spawn_view_with(
        commands,
        kit,
        background,
        Panel::NewWorldCustom,
        3.0,
        (5.0, 5.0),
        true,
        VIEW_HEIGHT - FOOTER,
    );

    commands.entity(content).with_children(|list| {
        illustration(list, kit, Scene::Dusk, Vec2::new(ROW.x, 115.8159), 138.1);

        panel(list, kit, Vec2::new(ROW.x, 44.9845), |row| {
            row.spawn((
                Blurb,
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(ROW.x * 0.5 - 19.9427 - 83.5),
                    width: px(167.0),
                    height: px(37.8),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|area| {
                area.spawn((
                    Pickable::IGNORE,
                    Text::new("Shape your world, then create it."),
                    kit.text(8.0),
                    TextColor(Color::WHITE),
                    TextLayout::justify(bevy::text::Justify::Center),
                    shadow(),
                ));
            });
            row.spawn((
                SizeWarning,
                Pickable::IGNORE,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(ROW.x * 0.5 - 19.9427 - 83.5),
                    width: px(167.0),
                    height: px(37.8),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|area| {
                area.spawn((
                    Pickable::IGNORE,
                    Text::new("Big worlds need a strong computer."),
                    kit.text(8.0),
                    TextColor(crate::tips::WARN),
                    TextLayout::justify(bevy::text::Justify::Center),
                    shadow(),
                ));
            });
            row.spawn((
                Preview,
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(192.51 - 22.49),
                    top: px(22.372 - 22.49),
                    width: px(44.985),
                    height: px(44.985),
                    ..default()
                },
                ImageNode::default(),
            ));
        });

        let size_icon = kit.glyphs.get(&Glyph::Globe).cloned().unwrap_or_default();
        list.spawn((
            SizeButton,
            Hovered::default(),
            Tip::plain(
                "World Size",
                "Click for a bigger world, right click for a smaller one.",
            ),
            UiTransform::default(),
            Node {
                width: px(ROW.x),
                height: px(ROW.y),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                image: kit.red_bar.clone(),
                image_mode: sliced(6.0),
                ..default()
            },
        ))
        .with_children(|button| {
            for left in [true, false] {
                button.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: if left { px(14.0 - 11.46) } else { Val::Auto },
                        right: if left { Val::Auto } else { px(14.234 - 11.46) },
                        width: px(22.919),
                        height: px(22.919),
                        ..default()
                    },
                    ImageNode::new(size_icon.clone()),
                ));
            }
            button
                .spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(ROW.x - 53.3605),
                        height: px(18.3484),
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
                        SizeLabel,
                        Pickable::IGNORE,
                        Text::new(""),
                        kit.text(8.0),
                        TextColor(Color::WHITE),
                        shadow(),
                    ));
                });
        });

        for id in ROW_ORDER {
            if let Some(setting) = setting(id) {
                setting_row(list, kit, setting);
            }
        }

        list.spawn((
            ResetButton,
            Hovered::default(),
            Tip::plain("Reset", "Put this template's settings back."),
            UiTransform::default(),
            Node {
                width: px(ROW.x),
                height: px(ROW.y),
                flex_shrink: 0.0,
                justify_content: JustifyContent::Center,
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
                Text::new("RESET TO DEFAULT"),
                kit.text(8.0),
                TextColor(Color::WHITE),
                shadow(),
            ));
        });
    });

    commands.entity(background).with_children(|frame| {
        frame
            .spawn((
                CreateButton,
                Hovered::default(),
                Tip::plain("Create World", "Make a new world with these settings."),
                UiTransform::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px((FRAME_WIDTH - ROW.x) * 0.5),
                    top: px(VIEW_TOP + VIEW_HEIGHT - FOOTER + 3.0),
                    width: px(ROW.x),
                    height: px(FOOTER - 6.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.red_bar.clone(),
                    image_mode: sliced(6.0),
                    ..default()
                },
            ))
            .with_children(|button| {
                button.spawn((
                    Pickable::IGNORE,
                    Text::new("Create World"),
                    kit.text(11.0),
                    TextColor(Color::WHITE),
                    shadow(),
                ));
            });
    });
}

fn setting_row(list: &mut ChildSpawnerCommands, kit: &Kit, setting: &'static MapGenSetting) {
    let id = setting.id;
    let mark = kit.gen_marks.get(id).cloned().unwrap_or_default();
    let tip = Tip::plain(setting_name(id), "Changes how the new world is shaped.");
    let middle = ROW.x * 0.5;

    list.spawn((
        SettingRow(id),
        Node {
            width: px(ROW.x),
            height: px(ROW.y),
            flex_shrink: 0.0,
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ))
    .with_children(|row| {
        if setting.is_switch {
            row.spawn((
                SwitchButton(id),
                Hovered::default(),
                tip.clone(),
                UiTransform::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(middle - 81.805 - SWITCH.x * 0.5),
                    top: px((ROW.y - SWITCH.y) * 0.5),
                    width: px(SWITCH.x),
                    height: px(SWITCH.y),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.red_bar.clone(),
                    image_mode: sliced(5.0),
                    ..default()
                },
            ))
            .with_children(|switch| {
                switch.spawn((
                    SwitchMark(id),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(3.0),
                        width: px(16.0),
                        height: px(16.0),
                        ..default()
                    },
                    ImageNode::new(kit.check_on.clone()),
                ));
                switch.spawn((
                    SwitchLabel(id),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(20.0),
                        width: px(SWITCH.x - 22.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Text::new("ON"),
                    kit.text(8.0),
                    TextColor(Color::WHITE),
                ));
            });
        } else {
            for (up, x) in [(false, -94.41), (true, -69.2)] {
                let image = kit
                    .gen_marks
                    .get(if up { "plus" } else { "minus" })
                    .cloned()
                    .unwrap_or_default();
                row.spawn((
                    StepButton(id, up),
                    Hovered::default(),
                    tip.clone(),
                    UiTransform::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(middle + x - STEP_BUTTON * 0.5),
                        top: px((ROW.y - STEP_BUTTON) * 0.5),
                        width: px(STEP_BUTTON),
                        height: px(STEP_BUTTON),
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
                .with_children(|button| {
                    button.spawn((
                        Pickable::IGNORE,
                        Node {
                            width: px(14.0),
                            height: px(14.0),
                            ..default()
                        },
                        ImageNode::new(image),
                    ));
                });
            }

            row.spawn((
                Counter(id),
                RowInk(id),
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0.0),
                    width: px(31.7803),
                    height: px(ROW.y),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|counter| {
                counter.spawn((
                    Counter(id),
                    RowInk(id),
                    Pickable::IGNORE,
                    Text::new("0"),
                    kit.text(9.0),
                    TextColor(Color::WHITE),
                    shadow(),
                ));
            });
        }

        row.spawn((
            RowInk(id),
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(middle - 46.1 - 10.2),
                top: px((ROW.y - 19.0) * 0.5),
                width: px(20.4112),
                height: px(19.0),
                ..default()
            },
            ImageNode::new(mark),
        ));

        row.spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(middle + 20.3568 - (ROW.x - 104.2736) * 0.5),
                width: px(ROW.x - 104.2736),
                height: px(ROW.y),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|text| {
            text.spawn((
                RowInk(id),
                Pickable::IGNORE,
                Text::new(setting_name(id)),
                kit.text(8.0),
                TextColor(Color::WHITE),
                shadow(),
            ));
        });
    });
}

pub fn refresh(
    draft: Res<WorldDraft>,
    kit: Option<Res<Kit>>,
    mut rows: Query<(&SettingRow, &mut Node)>,
    mut counters: Query<(&Counter, &mut Text), Without<SizeLabel>>,
    mut labels: Query<(&SwitchLabel, &mut Text), (Without<Counter>, Without<SizeLabel>)>,
    mut marks: Query<(&SwitchMark, &mut ImageNode), (Without<RowInk>, Without<Preview>)>,
    mut switches: Query<
        (&SwitchButton, &mut ImageNode),
        (Without<SwitchMark>, Without<RowInk>, Without<Preview>),
    >,
    mut inks: Query<
        (&RowInk, Option<&mut TextColor>, Option<&mut ImageNode>),
        (Without<SwitchButton>, Without<Preview>),
    >,
    mut sizes: Query<&mut Text, (With<SizeLabel>, Without<Counter>)>,
    mut warnings: Query<&mut Visibility, (With<SizeWarning>, Without<Blurb>)>,
    mut blurbs: Query<&mut Visibility, (With<Blurb>, Without<SizeWarning>)>,
    mut previews: Query<
        &mut ImageNode,
        (
            With<Preview>,
            Without<RowInk>,
            Without<SwitchMark>,
            Without<SwitchButton>,
        ),
    >,
    mut ready: Local<bool>,
) {
    if *ready && !draft.is_changed() {
        return;
    }
    let Some(kit) = kit else { return };
    *ready = true;

    let values = draft.config.values;

    for (row, mut node) in &mut rows {
        let display = match setting(row.0) {
            Some(setting) if draft.shown(setting) => Display::Flex,
            _ => Display::None,
        };
        if node.display != display {
            node.display = display;
        }
    }

    for (counter, mut text) in &mut counters {
        if let Some(setting) = setting(counter.0) {
            let wanted = setting.value(&values).to_string();
            if text.as_str() != wanted {
                **text = wanted;
            }
        }
    }

    for (label, mut text) in &mut labels {
        let on = setting(label.0).is_some_and(|s| s.value(&values) != 0);
        let wanted = if on { "ON" } else { "OFF" };
        if text.as_str() != wanted {
            **text = wanted.to_string();
        }
    }

    for (mark, mut image) in &mut marks {
        let on = setting(mark.0).is_some_and(|s| s.value(&values) != 0);
        image.image = if on {
            kit.check_on.clone()
        } else {
            kit.check_off.clone()
        };
    }

    for (switch, mut image) in &mut switches {
        let on = setting(switch.0).is_some_and(|s| s.value(&values) != 0);
        image.color = Color::srgba(1.0, 1.0, 1.0, if on { 1.0 } else { 0.8 });
    }

    for (ink, color, image) in &mut inks {
        let lit = setting(ink.0).is_some_and(|s| s.value(&values) != 0);
        let wanted = if lit { Color::WHITE } else { DISABLED };
        if let Some(mut color) = color {
            color.0 = wanted;
        }
        if let Some(mut image) = image {
            image.color = wanted;
        }
    }

    let size = draft.config.size;
    for mut text in &mut sizes {
        let wanted = format!("{} {}x{}", size_name(size), size.tiles(), size.tiles());
        if text.as_str() != wanted {
            **text = wanted;
        }
    }

    let warn = size.show_warning();
    for mut visibility in &mut warnings {
        visibility.set_if_neq(if warn {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for mut visibility in &mut blurbs {
        visibility.set_if_neq(if warn {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }

    if let Some(preview) = kit.template_previews.get(&draft.config.template) {
        for mut image in &mut previews {
            image.image = preview.clone();
        }
    }
}

pub fn size_name(size: MapSize) -> &'static str {
    match size {
        MapSize::Tiny => "Tiny",
        MapSize::Small => "Small",
        MapSize::Standard => "Standard",
        MapSize::Large => "Large",
        MapSize::Huge => "Huge",
        MapSize::Gigantic => "Gigantic",
        MapSize::Titanic => "Titanic",
        MapSize::Iceberg => "Iceberg",
    }
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    templates: Query<&TemplateButton>,
    steps: Query<&StepButton>,
    switches: Query<&SwitchButton>,
    sizes: Query<(), With<SizeButton>>,
    resets: Query<(), With<ResetButton>>,
    creates: Query<(), With<CreateButton>>,
    mut draft: ResMut<WorldDraft>,
    mut open: ResMut<OpenWindow>,
    mut exit: ResMut<Exit>,
    mut created: MessageWriter<CreateWorld>,
    mut cues: MessageWriter<Cue>,
) {
    let entity = click.entity;
    let primary = click.button == PointerButton::Primary;
    let secondary = click.button == PointerButton::Secondary;
    let many = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if sizes.contains(entity) && (primary || secondary) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        if primary {
            draft.grow();
        } else {
            draft.shrink();
        }
        return;
    }

    if !primary {
        return;
    }

    if let Ok(button) = templates.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        draft.pick(button.0);
        open.0 = Some(Panel::NewWorldCustom);
    } else if let Ok(step) = steps.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        if let Some(setting) = setting(step.0) {
            let values = &mut draft.config.values;
            if step.1 {
                setting.increase(values, many);
            } else {
                setting.decrease(values, many);
            }
        }
    } else if let Ok(switch) = switches.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        if let Some(setting) = setting(switch.0) {
            setting.switch(&mut draft.config.values);
        }
    } else if resets.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        draft.reset();
    } else if creates.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        created.write(CreateWorld(draft.config));
        exit.0 = Toward::Left;
        open.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_is_a_known_setting_and_none_is_left_out() {
        for id in ROW_ORDER {
            assert!(setting(id).is_some(), "{id}");
        }
        assert_eq!(ROW_ORDER.len(), econbox_worldgen::SETTINGS.len());
    }

    #[test]
    fn the_size_cycles_through_every_world_size() {
        assert_eq!(step_size(MapSize::Standard, 1), MapSize::Large);
        assert_eq!(step_size(MapSize::Tiny, -1), MapSize::Iceberg);
        assert_eq!(step_size(MapSize::Iceberg, 1), MapSize::Tiny);
    }

    #[test]
    fn each_template_keeps_its_own_edits_until_reset() {
        let mut draft = WorldDraft::default();
        let shapes = setting("gen_random_shapes").unwrap();
        shapes.increase(&mut draft.config.values, false);
        let edited = draft.config.values;

        draft.pick(TemplateId::Islands);
        assert_eq!(draft.config.values, TemplateId::Islands.template().values);

        draft.pick(TemplateId::Continent);
        assert_eq!(draft.config.values, edited);

        draft.reset();
        assert_eq!(draft.config.values, TemplateId::Continent.template().values);
    }

    #[test]
    fn a_preview_paints_every_cell() {
        let pixels = preview_pixels(TemplateId::Continent);
        assert_eq!(pixels.len(), (PREVIEW_SIDE * PREVIEW_SIDE) as usize);
        assert!(pixels.iter().any(|px| px[2] > px[0]));
    }
}
