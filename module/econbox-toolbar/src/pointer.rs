//! What the pointer carries: whether it rests on the interface, the chosen power's icon, and the brush outline on the map.

use crate::art::{Kit, crisp};
use crate::bar::px;
use crate::layout::ICON;
use crate::{Chosen, OpenWindow};
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use econbox_art::ui::Picture;
use econbox_brush::{Brush, PointerOverUi, mask};
use econbox_camera::TILE_SIZE;
use econbox_creature::centre_of;
use econbox_cursor::Hovered;
use econbox_tilemap::TileMap;

pub const GRACE: f32 = 0.05;

#[derive(Resource, Debug, Default)]
pub struct Grace(pub f32);

#[derive(Component)]
pub struct CarriedIcon;

#[derive(Component)]
pub struct CarriedShadow;

#[derive(Component, Default)]
pub struct Footprint {
    shape: Option<(econbox_brush::Shape, u32)>,
}

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        CarriedShadow,
        Pickable::IGNORE,
        GlobalZIndex(800),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: px(ICON),
            height: px(ICON),
            ..default()
        },
        ImageNode {
            color: Color::srgba(0.0, 0.0, 0.0, 0.3),
            ..default()
        },
    ));

    commands.spawn((
        CarriedIcon,
        Pickable::IGNORE,
        GlobalZIndex(801),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: px(ICON),
            height: px(ICON),
            ..default()
        },
        ImageNode::default(),
    ));

    commands.spawn((
        Footprint::default(),
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.22),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 5.0),
        Visibility::Hidden,
    ));
}

pub fn over_ui(
    time: Res<Time>,
    hovers: Option<Res<HoverMap>>,
    nodes: Query<(), With<ComputedNode>>,
    mut grace: ResMut<Grace>,
    mut over: ResMut<PointerOverUi>,
) {
    let touching = hovers.is_some_and(|hovers| {
        hovers
            .get(&PointerId::Mouse)
            .is_some_and(|hits| hits.keys().any(|entity| nodes.get(*entity).is_ok()))
    });

    grace.0 = if touching {
        GRACE
    } else {
        (grace.0 - time.delta_secs()).max(0.0)
    };

    let wanted = touching || grace.0 > 0.0;

    if over.0 != wanted {
        over.0 = wanted;
    }
}

pub fn follow(
    chosen: Res<Chosen>,
    over: Res<PointerOverUi>,
    open: Res<OpenWindow>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    kit: Option<Res<Kit>>,
    mut icons: Query<
        (&mut Node, &mut Visibility, &mut ImageNode),
        (With<CarriedIcon>, Without<CarriedShadow>),
    >,
    mut shadows: Query<
        (&mut Node, &mut Visibility, &mut ImageNode),
        (With<CarriedShadow>, Without<CarriedIcon>),
    >,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let cursor = window.cursor_position().map(|at| at / scale.0);
    let def = chosen.def();
    let carried = def.filter(|def| def.id != "clock" && def.id != "pause");
    let visible = carried.is_some()
        && cursor.is_some()
        && !over.0
        && open.0.is_none()
        && buttons.get_pressed().next().is_none();

    let image = match (&kit, carried) {
        (Some(kit), Some(def)) => kit.icons.get(def.id).cloned(),
        _ => None,
    };

    let at = cursor.unwrap_or_default();
    let offset = Vec2::new(4.0, -ICON - 4.0);

    for (mut node, mut seen, mut icon) in &mut icons {
        seen.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });

        if let Some(image) = &image
            && icon.image != *image
        {
            icon.image = image.clone();
        }

        node.left = px(at.x + offset.x);
        node.top = px(at.y + offset.y);
    }

    for (mut node, mut seen, mut icon) in &mut shadows {
        seen.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });

        if let Some(image) = &image
            && icon.image != *image
        {
            icon.image = image.clone();
        }

        node.left = px(at.x + offset.x + 1.5);
        node.top = px(at.y + offset.y + 1.5);
    }
}

pub fn footprint(
    chosen: Res<Chosen>,
    brush: Res<Brush>,
    over: Res<PointerOverUi>,
    open: Res<OpenWindow>,
    hovered: Res<Hovered>,
    map: Option<Res<TileMap>>,
    images: Option<ResMut<Assets<Image>>>,
    mut prints: Query<(&mut Footprint, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    let wanted = chosen.def().is_some_and(|def| def.sizes) && !brush.flood;
    let (Some(map), Some(mut images)) = (map, images) else {
        return;
    };

    for (mut print, mut sprite, mut transform, mut visibility) in &mut prints {
        let at = hovered
            .pos
            .filter(|_| wanted && !over.0 && open.0.is_none());

        let Some(pos) = at else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };

        let shape = (brush.shape, brush.radius);

        if print.shape != Some(shape) {
            let (side, cells) = mask(brush.shape, brush.radius);
            let mut picture = Picture::blank(side, side);

            for y in 0..side {
                for x in 0..side {
                    if cells[(y * side + x) as usize] {
                        picture.set(x, y, [255, 255, 255, 255]);
                    }
                }
            }

            sprite.image = images.add(crisp(&picture));
            sprite.custom_size = Some(Vec2::splat(side as f32 * TILE_SIZE));
            print.shape = Some(shape);
        }

        let centre = centre_of(&map, pos);
        transform.translation.x = centre.x;
        transform.translation.y = centre.y;
        visibility.set_if_neq(Visibility::Inherited);
    }
}
