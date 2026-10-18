//! The settings window: its tabs, the illustrated sections, and a row for every option.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::options::{Control, Options, SETTINGS, Section, Setting, Value, setting};
use crate::press::press_pop;
use crate::sound::Cue;
use crate::tips::{TITLE, Tip};
use crate::windows::{
    CONTENT_WIDTH, TAB_CELL, Viewport, shadow, spawn_frame, spawn_tab_column, spawn_view,
};
use crate::{OpenWindow, Panel};
use bevy::picking::events::{PointerClick, PointerPress};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use econbox_art::Glyph;
use econbox_art::window::Scene;

pub const ROW_WIDTH: f32 = 200.0;
pub const AREA_HEIGHT: f32 = 25.0;
pub const SWITCH_WIDTH: f32 = 37.0;
pub const SWITCH_HEIGHT: f32 = 18.0;
pub const SLIDER_HEIGHT: f32 = 20.0;
pub const KNOB: f32 = 20.0;
pub const DISABLED: Color = Color::srgb(0.8, 0.8, 0.8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tab {
    #[default]
    Everything,
    General,
    Interface,
    Graphics,
    Other,
}

pub const TABS: [Tab; 5] = [
    Tab::Everything,
    Tab::General,
    Tab::Interface,
    Tab::Graphics,
    Tab::Other,
];

impl Tab {
    pub fn name(self) -> &'static str {
        match self {
            Self::Everything => "Everything",
            Self::General => "General",
            Self::Interface => "Interface",
            Self::Graphics => "Graphics",
            Self::Other => "Other Settings",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Self::Everything => "Every option on one page.",
            Self::General => "Screen and sound.",
            Self::Interface => "Sizes of the interface.",
            Self::Graphics => "Effects drawn over the world.",
            Self::Other => "Everything else.",
        }
    }

    pub fn glyph(self) -> Glyph {
        match self {
            Self::Everything => Glyph::Info,
            Self::General => Glyph::Globe,
            Self::Interface => Glyph::Bubbles,
            Self::Graphics => Glyph::Gem,
            Self::Other => Glyph::Gear,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SettingsTab(pub Tab);

#[derive(Component, Debug, Clone, Copy)]
pub struct ShownIn(pub &'static [Tab]);

#[derive(Component, Debug, Clone, Copy)]
pub struct TabButton(pub Tab);

#[derive(Component, Debug, Clone, Copy)]
pub struct Switch(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchLabel(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchMark(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct RowInk(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct Counter(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct Slider(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct SliderFill(pub &'static str);

#[derive(Component, Debug, Clone, Copy)]
pub struct SliderKnob(pub &'static str);

#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct LanguageButton;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SliderDrag(pub Option<&'static str>);

const EVERYTHING: &[Tab] = &[Tab::Everything];
const MAIN: &[Tab] = &[Tab::Everything, Tab::General];
const GENERAL: &[Tab] = &[Tab::General];
const INTERFACE: &[Tab] = &[Tab::Everything, Tab::Interface];
const INTERFACE_ONLY: &[Tab] = &[Tab::Interface];
const GRAPHICS: &[Tab] = &[Tab::Everything, Tab::Graphics];
const GRAPHICS_ONLY: &[Tab] = &[Tab::Graphics];
const OTHER: &[Tab] = &[Tab::Everything, Tab::Other];
const OTHER_ONLY: &[Tab] = &[Tab::Other];

pub fn tip_for(setting: &Setting) -> Tip {
    let mut tip = Tip::plain(setting.name, setting.about);
    if let Some(system) = setting.missing {
        tip.foot = Some(format!("Not available yet: needs {system}."));
        tip.warn = true;
    }
    tip
}

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };
    let (_, background) = spawn_frame(
        &mut commands,
        &kit,
        Panel::Settings,
        kit.crab.clone(),
        Vec2::new(17.12, 320.0 - 313.22),
    );

    let column = spawn_tab_column(&mut commands, background);
    commands.entity(column).with_children(|tabs| {
        for tab in TABS {
            let icon = kit.glyphs.get(&tab.glyph()).cloned().unwrap_or_default();
            tabs.spawn((
                TabButton(tab),
                Hovered::default(),
                Tip::plain(tab.name(), tab.about()),
                Node {
                    width: px(TAB_CELL),
                    height: px(TAB_CELL),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.side_tab.clone(),
                    image_mode: sliced(6.0),
                    ..default()
                },
            ))
            .with_children(|button| {
                button.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(28.0 * 0.65),
                        height: px(28.0 * 0.65),
                        margin: UiRect::left(px(2.08)),
                        ..default()
                    },
                    ImageNode::new(icon),
                ));
            });
        }
    });

    let content = spawn_view(
        &mut commands,
        &kit,
        background,
        Panel::Settings,
        2.6,
        (3.0, 10.0),
        true,
    );

    commands.entity(content).with_children(|list| {
        illustration(list, &kit, Scene::Ember, 180.0, MAIN);
        band(list, &kit, Tab::General, GENERAL);
        language_button(list, &kit);

        illustration(list, &kit, Scene::Dusk, 206.0, INTERFACE_ONLY);
        section(
            list,
            &kit,
            Section::Interface,
            Some((Tab::Interface, INTERFACE_ONLY)),
            INTERFACE,
            5.0,
        );
        section(list, &kit, Section::Video, None, MAIN, 0.0);
        section(list, &kit, Section::Audio, None, MAIN, 5.0);

        illustration(list, &kit, Scene::Grove, 206.0, GRAPHICS_ONLY);
        section(
            list,
            &kit,
            Section::Graphics,
            Some((Tab::Graphics, GRAPHICS_ONLY)),
            GRAPHICS,
            5.0,
        );

        illustration(list, &kit, Scene::Tide, 206.0, OTHER_ONLY);
        section(
            list,
            &kit,
            Section::Other,
            Some((Tab::Other, OTHER_ONLY)),
            OTHER,
            5.0,
        );
    });
}

fn illustration(
    list: &mut ChildSpawnerCommands,
    kit: &Kit,
    scene: Scene,
    height: f32,
    tabs: &'static [Tab],
) {
    let art = kit.scenes.get(&scene).cloned().unwrap_or_default();
    list.spawn((
        ShownIn(tabs),
        Pickable::IGNORE,
        Node {
            width: px(CONTENT_WIDTH),
            height: px(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
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
                height: px(height - 6.0),
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

fn band(list: &mut ChildSpawnerCommands, kit: &Kit, tab: Tab, tabs: &'static [Tab]) {
    let icon = kit.glyphs.get(&tab.glyph()).cloned().unwrap_or_default();
    list.spawn((
        ShownIn(tabs),
        Pickable::IGNORE,
        Node {
            width: px(214.0),
            height: px(22.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        },
        ImageNode {
            image: kit.band.clone(),
            image_mode: sliced(6.0),
            ..default()
        },
    ))
    .with_children(|band| {
        for left in [true, false] {
            band.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: if left { px(11.0 - 7.0) } else { Val::Auto },
                    right: if left { Val::Auto } else { px(11.0 - 7.0) },
                    width: px(14.0),
                    height: px(14.0),
                    ..default()
                },
                ImageNode::new(icon.clone()),
            ));
        }

        band.spawn((
            Pickable::IGNORE,
            Text::new(tab.name()),
            TextFont::from_font_size(9.0),
            TextColor(TITLE),
        ));
    });
}

fn language_button(list: &mut ChildSpawnerCommands, kit: &Kit) {
    let icon = kit.glyphs.get(&Glyph::Bubbles).cloned().unwrap_or_default();
    let mut tip = Tip::plain("Change Language", "Pick the language of the interface.");
    tip.foot = Some("Not available yet: needs translations.".to_string());
    tip.warn = true;

    list.spawn((
        ShownIn(EVERYTHING),
        LanguageButton,
        Hovered::default(),
        tip,
        UiTransform::default(),
        Node {
            width: px(CONTENT_WIDTH),
            height: px(23.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
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
                    left: if left { px(14.0 - 9.8) } else { Val::Auto },
                    right: if left { Val::Auto } else { px(14.0 - 9.8) },
                    width: px(19.6),
                    height: px(19.6),
                    ..default()
                },
                ImageNode::new(icon.clone()),
            ));
        }

        button.spawn((
            Pickable::IGNORE,
            Text::new("Change Language"),
            TextFont::from_font_size(12.0),
            TextColor(Color::WHITE),
        ));
    });
}

fn section(
    list: &mut ChildSpawnerCommands,
    kit: &Kit,
    section: Section,
    banded: Option<(Tab, &'static [Tab])>,
    tabs: &'static [Tab],
    padding: f32,
) {
    list.spawn((
        ShownIn(tabs),
        Node {
            width: px(CONTENT_WIDTH),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(5.0),
            padding: UiRect::bottom(px(padding)),
            flex_shrink: 0.0,
            ..default()
        },
    ))
    .with_children(|body| {
        let title_tabs = match banded {
            Some((tab, only)) => {
                band(body, kit, tab, only);
                EVERYTHING
            }
            None => tabs,
        };

        body.spawn((
            ShownIn(title_tabs),
            Pickable::IGNORE,
            Node {
                height: px(14.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Text::new(section.title()),
            TextFont::from_font_size(10.0),
            TextColor(Color::WHITE),
            shadow(),
        ));

        body.spawn((
            Node {
                width: px(CONTENT_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(3.66),
                padding: UiRect::all(px(5.0)),
                ..default()
            },
            ImageNode {
                image: kit.inset.clone(),
                image_mode: sliced(4.0),
                ..default()
            },
        ))
        .with_children(|grid| {
            for setting in SETTINGS.iter().filter(|setting| setting.section == section) {
                row(grid, kit, setting);
            }

            if section == Section::Other {
                reset_button(grid, kit);
            }
        });
    });
}

fn row(grid: &mut ChildSpawnerCommands, kit: &Kit, setting: &'static Setting) {
    let id = setting.id;
    let mark = kit.option_marks.get(id).cloned().unwrap_or_default();
    let toggle = matches!(setting.control, Control::Toggle(_));

    grid.spawn((
        Node {
            width: px(ROW_WIDTH),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ))
    .with_children(|row| {
        row.spawn(Node {
            height: px(AREA_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(5.0)),
            column_gap: px(3.0),
            ..default()
        })
        .with_children(|area| {
            if toggle {
                area.spawn((
                    Switch(id),
                    Hovered::default(),
                    tip_for(setting),
                    UiTransform::default(),
                    Node {
                        width: px(SWITCH_WIDTH),
                        height: px(SWITCH_HEIGHT),
                        flex_shrink: 0.0,
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
                            left: px(SWITCH_WIDTH * 0.5 - 13.7 - 8.0),
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
                            left: px(SWITCH_WIDTH * 0.5 + 7.92 - 12.76),
                            width: px(25.52),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Text::new("ON"),
                        TextFont::from_font_size(8.0),
                        TextColor(Color::WHITE),
                    ));
                });
            }

            area.spawn((
                RowInk(id),
                Pickable::IGNORE,
                Node {
                    width: px(20.0),
                    height: px(20.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(mark),
            ));

            area.spawn((
                RowInk(id),
                Pickable::IGNORE,
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
                Text::new(setting.name),
                TextFont::from_font_size(9.0),
                TextColor(Color::WHITE),
                shadow(),
            ));

            if !toggle {
                area.spawn((
                    Counter(id),
                    RowInk(id),
                    Pickable::IGNORE,
                    Text::new(""),
                    TextFont::from_font_size(8.0),
                    TextColor(Color::WHITE),
                    shadow(),
                ));
            }
        });

        if toggle {
            return;
        }

        row.spawn(Node {
            height: px(AREA_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|area| {
            area.spawn((
                Slider(id),
                Hovered::default(),
                tip_for(setting),
                Node {
                    width: px(ROW_WIDTH - 10.0),
                    height: px(SLIDER_HEIGHT),
                    ..default()
                },
            ))
            .with_children(|slider| {
                slider.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        right: px(0.0),
                        top: px(SLIDER_HEIGHT * 0.25),
                        bottom: px(SLIDER_HEIGHT * 0.25),
                        ..default()
                    },
                    ImageNode {
                        image: kit.gray_bar.clone(),
                        image_mode: sliced(3.0),
                        ..default()
                    },
                ));
                slider.spawn((
                    SliderFill(id),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        width: px(KNOB),
                        top: px(SLIDER_HEIGHT * 0.25),
                        bottom: px(SLIDER_HEIGHT * 0.25),
                        ..default()
                    },
                    ImageNode {
                        image: kit.red_bar.clone(),
                        image_mode: sliced(4.0),
                        ..default()
                    },
                ));
                slider.spawn((
                    SliderKnob(id),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: px(KNOB),
                        height: px(SLIDER_HEIGHT),
                        ..default()
                    },
                    ImageNode {
                        image: kit.red_bar.clone(),
                        image_mode: sliced(5.0),
                        ..default()
                    },
                ));
            });
        });
    });
}

fn reset_button(grid: &mut ChildSpawnerCommands, kit: &Kit) {
    grid.spawn((
        ResetButton,
        Hovered::default(),
        Tip::plain(
            "Reset to Default",
            "Put every option back the way it started.",
        ),
        UiTransform::default(),
        Node {
            width: px(ROW_WIDTH),
            height: px(25.0),
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
                    left: if left { px(15.0 - 11.46) } else { Val::Auto },
                    right: if left { Val::Auto } else { px(15.0 - 11.46) },
                    width: px(22.919),
                    height: px(22.919),
                    ..default()
                },
                ImageNode::new(kit.undo.clone()),
            ));
        }

        button
            .spawn((
                Pickable::IGNORE,
                Node {
                    width: px(ROW_WIDTH - 55.3345),
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
                    Pickable::IGNORE,
                    Text::new("RESET TO DEFAULT"),
                    TextFont::from_font_size(8.0),
                    TextColor(Color::WHITE),
                ));
            });
    });
}

pub fn show_tabs(
    tab: Res<SettingsTab>,
    open: Res<OpenWindow>,
    kit: Option<Res<Kit>>,
    mut nodes: Query<(&ShownIn, &mut Node)>,
    mut buttons: Query<(&TabButton, &mut ImageNode)>,
    mut views: Query<(&Viewport, &mut ScrollPosition)>,
    mut last: Local<Option<Tab>>,
) {
    let fresh = *last != Some(tab.0) || open.is_changed() && open.0 == Some(Panel::Settings);
    if !fresh {
        return;
    }
    *last = Some(tab.0);

    for (shown, mut node) in &mut nodes {
        let display = if shown.0.contains(&tab.0) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }

    for (view, mut position) in &mut views {
        if view.0 == Panel::Settings {
            position.y = 0.0;
        }
    }

    let Some(kit) = kit else { return };
    for (button, mut image) in &mut buttons {
        let wanted = if button.0 == tab.0 {
            &kit.side_tab_on
        } else {
            &kit.side_tab
        };
        if image.image != *wanted {
            image.image = wanted.clone();
        }
    }
}

pub fn refresh(
    options: Res<Options>,
    kit: Option<Res<Kit>>,
    mut switches: Query<(&Switch, &mut ImageNode), Without<SwitchMark>>,
    mut marks: Query<(&SwitchMark, &mut ImageNode), Without<Switch>>,
    mut labels: Query<(&SwitchLabel, &mut Text), Without<Counter>>,
    mut counters: Query<(&Counter, &mut Text), Without<SwitchLabel>>,
    mut inks: Query<
        (&RowInk, Option<&mut TextColor>, Option<&mut ImageNode>),
        (Without<Switch>, Without<SwitchMark>),
    >,
    mut fills: Query<(&SliderFill, &mut Node), Without<SliderKnob>>,
    mut knobs: Query<(&SliderKnob, &mut Node), Without<SliderFill>>,
    mut ready: Local<bool>,
) {
    if !options.is_changed() && *ready {
        return;
    }
    let Some(kit) = kit else { return };
    *ready = true;

    for (switch, mut image) in &mut switches {
        let alpha = if options.on(switch.0) { 1.0 } else { 0.8 };
        image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }

    for (mark, mut image) in &mut marks {
        let on = options.on(mark.0);
        image.image = if on {
            kit.check_on.clone()
        } else {
            kit.check_off.clone()
        };
        image.color = Color::srgba(1.0, 1.0, 1.0, if on { 1.0 } else { 0.8 });
    }

    for (label, mut text) in &mut labels {
        let wanted = if options.on(label.0) { "ON" } else { "OFF" };
        if text.as_str() != wanted {
            **text = wanted.to_string();
        }
    }

    for (counter, mut text) in &mut counters {
        let Some(setting) = setting(counter.0) else {
            continue;
        };
        let wanted = setting.counter(options.get(counter.0));
        if text.as_str() != wanted {
            **text = wanted;
        }
    }

    for (ink, color, image) in &mut inks {
        let lit = options.on(ink.0);
        let wanted = if lit { Color::WHITE } else { DISABLED };
        if let Some(mut color) = color {
            color.0 = wanted;
        }
        if let Some(mut image) = image {
            image.color = wanted;
        }
    }

    for (fill, mut node) in &mut fills {
        let at = fraction(fill.0, &options);
        node.width = px(KNOB * 0.5 + (ROW_WIDTH - 10.0 - KNOB) * at);
    }

    for (knob, mut node) in &mut knobs {
        let at = fraction(knob.0, &options);
        node.left = px((ROW_WIDTH - 10.0 - KNOB) * at);
    }
}

pub fn fraction(id: &str, options: &Options) -> f32 {
    match setting(id).map(|setting| setting.control) {
        Some(Control::Slider { min, max, .. }) => {
            (options.int(id) - min) as f32 / (max - min).max(1) as f32
        }
        _ => 0.0,
    }
}

pub fn value_at(id: &str, along: f32) -> Option<i32> {
    match setting(id)?.control {
        Control::Slider { min, max, .. } => {
            let t = along.clamp(0.0, 1.0);
            Some(min + ((max - min) as f32 * t) as i32)
        }
        Control::Toggle(_) => None,
    }
}

pub fn drag_slider(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut drag: ResMut<SliderDrag>,
    sliders: Query<(&Slider, &UiGlobalTransform, &ComputedNode)>,
    mut options: ResMut<Options>,
) {
    let Some(id) = drag.0 else { return };

    if !buttons.pressed(MouseButton::Left) {
        drag.0 = None;
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };

    for (slider, transform, computed) in &sliders {
        if slider.0 != id {
            continue;
        }

        let width = computed.size().x;
        let knob = KNOB / computed.inverse_scale_factor().max(f32::EPSILON) * 1.0;
        let left = transform.translation.x - width * 0.5 + knob * 0.5;
        let along = (cursor.x - left) / (width - knob).max(1.0);

        if let Some(value) = value_at(id, along)
            && options.int(id) != value
        {
            options.set(id, Value::Int(value));
        }
    }
}

pub fn on_press(
    mut press: On<PointerPress>,
    sliders: Query<&Slider>,
    mut drag: ResMut<SliderDrag>,
) {
    if press.button != PointerButton::Primary {
        return;
    }

    if let Ok(slider) = sliders.get(press.entity) {
        press.propagate(false);
        drag.0 = Some(slider.0);
    }
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    tabs: Query<&TabButton>,
    switches: Query<&Switch>,
    resets: Query<(), With<ResetButton>>,
    languages: Query<(), With<LanguageButton>>,
    mut current: ResMut<SettingsTab>,
    mut options: ResMut<Options>,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if let Ok(tab) = tabs.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        if current.0 != tab.0 {
            current.0 = tab.0;
        }
    } else if let Ok(switch) = switches.get(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        options.flip(switch.0);
    } else if resets.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
        options.reset();
    } else if languages.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        commands.entity(entity).insert(press_pop());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slider_maps_its_track_onto_the_option_range() {
        assert_eq!(value_at("ui_size_main", 0.0), Some(50));
        assert_eq!(value_at("ui_size_main", 0.5), Some(100));
        assert_eq!(value_at("ui_size_main", 1.0), Some(150));
        assert_eq!(value_at("ui_size_main", 7.0), Some(150));
        assert_eq!(value_at("fire", 0.5), None);

        let options = Options::default();
        assert_eq!(fraction("ui_size_main", &options), 0.5);
        assert_eq!(fraction("volume_ui", &options), 1.0);
    }

    #[test]
    fn options_without_a_system_say_what_they_wait_for() {
        let fire = tip_for(setting("fire").unwrap());
        assert!(fire.warn);
        assert!(fire.foot.unwrap().contains("fire"));
        assert!(!tip_for(setting("ui_size_main").unwrap()).warn);
    }
}
