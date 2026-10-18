//! The column of window buttons that slides in on the left while a window is open.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::catalog::{Def, find};
use crate::motion::Mover;
use crate::options::Options;
use crate::press::press_pop;
use crate::size::{UiLayer, layer_scale, window_size};
use crate::sound::Cue;
use crate::tips::{Hover, Tip};
use crate::windows::{Exit, Toward};
use crate::{OpenWindow, Panel};
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::picking::events::PointerClick;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};
use bevy::ui::widget::NodeImageMode;

pub const WIDTH: f32 = 67.0;
pub const HEIGHT: f32 = 320.32;
pub const SHOWN_X: f32 = 25.6;
pub const HIDDEN_X: f32 = -42.0;
pub const DROP: f32 = 6.0;
pub const CONTENT_SCALE: f32 = 0.7;
pub const CELL: f32 = 32.0;
pub const GAP: f32 = 4.0;
pub const WHEEL_STEP: f32 = 20.0;
pub const HIDE_NEAR: f32 = 225.0;
pub const HIDE_FAR: f32 = 440.0;
pub const WINDOWS_MIN: f32 = 0.3;
pub const WINDOWS_MAX: f32 = 1.15;
pub const SELECTOR: f32 = 40.0;

pub const GROUPS: [&[&str]; 7] = [
    &["favorites_list", "favorites_items_list"],
    &["wars_list", "plots_list"],
    &["alliances_list", "armies_list", "kingdoms", "villages"],
    &[
        "clans_list",
        "religions_list",
        "cultures_list",
        "languages_list",
        "subspecies_list",
        "families",
    ],
    &["world_laws", "ages", "world_info", "world_log"],
    &["player_worlds", "custom_world_button"],
    &[
        "settings_button",
        "milestones",
        "statistics",
        "chart_comparer",
    ],
];

#[derive(Component)]
pub struct Sidebar;

#[derive(Component)]
pub struct SidebarView;

#[derive(Component, Debug, Clone, Copy)]
pub struct SidebarButton(pub &'static Def);

#[derive(Component)]
pub struct Selector;

pub fn opens(id: &str) -> Option<Panel> {
    match id {
        "settings_button" => Some(Panel::Other),
        _ => None,
    }
}

pub fn hide_distance(windows_percent: i32, screen_height: f32) -> f32 {
    let t = ((windows_percent as f32 / 100.0 - WINDOWS_MIN) / (WINDOWS_MAX - WINDOWS_MIN))
        .clamp(0.0, 1.0);
    (HIDE_NEAR + (HIDE_FAR - HIDE_NEAR) * t) * screen_height / 1080.0
}

pub fn too_close(physical: Vec2, scale: f32, windows_percent: i32) -> bool {
    let gap = physical.x * 0.5 - 50.0 * scale;
    gap < hide_distance(windows_percent, physical.y)
}

fn frame_slices() -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(24.0, 48.0),
            max_inset: Vec2::new(24.0, 54.0),
        },
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

pub fn spawn(mut commands: Commands, kit: Option<Res<Kit>>) {
    let Some(kit) = kit else { return };
    let s = CONTENT_SCALE;
    let view_top = 6.896 * s;
    let view_height = (452.0 - 18.6687) * s;
    let column = (CELL * 2.0 + GAP) * s;

    commands
        .spawn((
            Sidebar,
            UiLayer::Windows,
            GlobalZIndex(550),
            Mover::new(Vec2::new(HIDDEN_X - SHOWN_X, 0.0), false),
            UiTransform::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(SHOWN_X - WIDTH * 0.5),
                top: Val::Percent(50.0),
                margin: UiRect::top(px(-HEIGHT * 0.5 + DROP)),
                width: px(WIDTH),
                height: px(HEIGHT),
                ..default()
            },
            ImageNode {
                image: kit.sidebar.clone(),
                image_mode: frame_slices(),
                ..default()
            },
        ))
        .with_children(|bar| {
            bar.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(WIDTH * 0.5 + (54.587 - 4.3 - 18.0) * s),
                    top: px(HEIGHT * 0.5 - (3.2 + 195.6) * s),
                    width: px(36.0 * s),
                    height: px(391.2 * s),
                    ..default()
                },
                ImageNode {
                    image: kit.track.clone(),
                    image_mode: sliced(4.0),
                    ..default()
                },
            ))
            .with_children(|track| {
                track.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px((36.0 - 6.0) * s * 0.5 + 1.0),
                        top: px(11.5 * s),
                        bottom: px(11.5 * s),
                        width: px(6.0 * s),
                        ..default()
                    },
                    ImageNode {
                        image: kit.handle.clone(),
                        image_mode: sliced(2.0),
                        color: Color::srgb(0.906, 0.325, 0.251),
                        ..default()
                    },
                ));
            });

            bar.spawn((
                SidebarView,
                ScrollPosition::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px((WIDTH - 86.0 * s) * 0.5),
                    top: px((HEIGHT - 452.0 * s) * 0.5 + view_top),
                    width: px(86.0 * s),
                    height: px(view_height),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(1.0 * s),
                    ..default()
                },
            ))
            .with_children(|view| {
                for (at, group) in GROUPS.iter().enumerate() {
                    if at > 0 {
                        view.spawn((
                            Pickable::IGNORE,
                            Node {
                                width: px(column),
                                height: px(2.0 * s),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ImageNode {
                                image: kit.line.clone(),
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                        ));
                    }

                    view.spawn(Node {
                        width: px(column),
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::px(2, CELL * s),
                        column_gap: px(GAP * s),
                        row_gap: px(GAP * s),
                        flex_shrink: 0.0,
                        ..default()
                    })
                    .with_children(|grid| {
                        for id in group.iter() {
                            let Some(def) = find(id) else { continue };
                            spawn_button(grid, &kit, def);
                        }
                    });
                }
            });
        });

    commands.spawn((
        Selector,
        UiLayer::Windows,
        Pickable::IGNORE,
        GlobalZIndex(560),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: px(SELECTOR * s),
            height: px(SELECTOR * s),
            ..default()
        },
        ImageNode {
            image: kit.selection.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ));
}

fn spawn_button(grid: &mut ChildSpawnerCommands, kit: &Kit, def: &'static Def) {
    let s = CONTENT_SCALE;
    let icon = kit.icons.get(def.id).cloned().unwrap_or_default();
    let tint = if opens(def.id).is_some() {
        Color::WHITE
    } else {
        Color::srgba(0.6, 0.6, 0.6, 0.9)
    };

    grid.spawn((
        SidebarButton(def),
        Hovered::default(),
        Tip::of(def),
        UiTransform::default(),
        Node {
            width: px(CELL * s),
            height: px(CELL * s),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode {
            image: kit.plate(def.plate).clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
    ))
    .with_children(|button| {
        button.spawn((
            Pickable::IGNORE,
            Node {
                width: px(28.0 * s),
                height: px(28.0 * s),
                ..default()
            },
            ImageNode {
                image: icon,
                color: tint,
                ..default()
            },
        ));
    });
}

pub fn show(
    open: Res<OpenWindow>,
    windows: Query<&Window>,
    options: Res<Options>,
    mut sidebars: Query<&mut Mover, With<Sidebar>>,
) {
    let visible = open.0.is_some()
        && windows.iter().next().is_none_or(|window| {
            let size = window_size(window);
            let scale = layer_scale(UiLayer::Windows, size, &options) * window.scale_factor();
            let physical = Vec2::new(
                window.physical_width() as f32,
                window.physical_height() as f32,
            );
            !too_close(physical, scale, options.int("ui_size_windows"))
        });

    for mut mover in &mut sidebars {
        mover.show(visible);
    }
}

pub fn select(
    open: Res<OpenWindow>,
    buttons: Query<(&SidebarButton, &UiGlobalTransform, &ComputedNode)>,
    mut selectors: Query<(&mut Node, &mut Visibility, &ComputedNode), With<Selector>>,
) {
    let target = open.0.and_then(Panel::sidebar_button);

    for (mut node, mut visibility, own) in &mut selectors {
        let found = target.and_then(|id| {
            buttons.iter().find(|(button, _, _)| button.0.id == id).map(
                |(_, transform, computed)| (transform.translation, computed.inverse_scale_factor()),
            )
        });

        let Some((centre, inverse)) = found else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };

        let half = own.size() * own.inverse_scale_factor() * 0.5;
        let at = centre * inverse - half;
        let (left, top) = (px(at.x), px(at.y));
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}

pub fn wheel(
    open: Res<OpenWindow>,
    wheel: Res<AccumulatedMouseScroll>,
    hover: Res<Hover>,
    buttons: Query<(), With<SidebarButton>>,
    mut views: Query<(&mut ScrollPosition, &ComputedNode), With<SidebarView>>,
) {
    if open.0.is_none() || wheel.delta.y == 0.0 {
        return;
    }

    if !hover.target.is_some_and(|target| buttons.contains(target)) {
        return;
    }

    let step = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * WHEEL_STEP * CONTENT_SCALE,
        MouseScrollUnit::Pixel => wheel.delta.y,
    };

    for (mut position, computed) in &mut views {
        let limit = crate::windows::scroll_limit(computed);
        position.y = (position.y - step).clamp(0.0, limit);
    }
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    buttons: Query<&SidebarButton>,
    mut open: ResMut<OpenWindow>,
    mut exit: ResMut<Exit>,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let Ok(button) = buttons.get(click.entity) else {
        return;
    };
    click.propagate(false);
    commands.entity(click.entity).insert(press_pop());
    cues.write(Cue::Click);

    let Some(panel) = opens(button.0.id) else {
        return;
    };

    if open.0 != Some(panel) {
        exit.0 = Toward::Right;
        open.0 = Some(panel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sidebar_button_is_in_the_catalog() {
        for group in GROUPS {
            for id in group.iter() {
                assert!(find(id).is_some(), "{id}");
            }
        }
        assert_eq!(GROUPS.iter().map(|group| group.len()).sum::<usize>(), 24);
    }

    #[test]
    fn the_sidebar_hides_when_a_window_crowds_the_left_edge() {
        let screen = Vec2::new(2560.0, 1440.0);
        let scale = 1440.0 / 420.0;
        assert!(!too_close(screen, scale, 100));
        assert!(too_close(Vec2::new(1000.0, 1440.0), scale, 100));

        let near = hide_distance(30, 1080.0);
        let far = hide_distance(115, 1080.0);
        assert_eq!(near, HIDE_NEAR);
        assert!((far - HIDE_FAR).abs() < 1e-3);
    }

    #[test]
    fn only_the_settings_button_opens_a_window_so_far() {
        assert_eq!(opens("settings_button"), Some(Panel::Other));
        assert_eq!(opens("kingdoms"), None);
    }
}
