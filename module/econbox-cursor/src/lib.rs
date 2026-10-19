//! The pointer: the tile it rests on, WorldBox's animated hand cursor, and how fast it moves while drawing.

mod art;
mod look;
mod speed;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow};
use econbox_camera::{Panning, PointerOverUi, TILE_SIZE, WorldCamera};
use econbox_tile::Tile;
use econbox_tilemap::{TileMap, TilePos};

pub use art::{HOTSPOT, SIDE, draw as draw_look, rgba as look_rgba};
pub use look::{
    ATTACK, CursorHints, CursorLook, DEFAULT, DOWN, DRAG, DRAW, HOLD, HoldAnimation, INSPECT,
    LOOKS, LookInput, OVER_UI, PINKIE, SPRINKLE, UP,
};
pub use speed::CursorSpeed;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Hovered {
    pub pos: Option<TilePos>,
    pub tile: Option<Tile>,
}

#[derive(Resource, Debug, Clone)]
pub struct CursorImages(pub Vec<Handle<Image>>);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CursorSystems;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hovered>()
            .init_resource::<CursorHints>()
            .init_resource::<CursorLook>()
            .init_resource::<CursorSpeed>()
            .add_systems(Startup, forge)
            .add_systems(
                Update,
                (
                    track.run_if(resource_exists::<TileMap>),
                    pace,
                    animate,
                    wear.run_if(resource_exists::<CursorImages>),
                )
                    .chain()
                    .in_set(CursorSystems),
            );
    }
}

pub fn tile_at(map: &TileMap, world: Vec2) -> Option<TilePos> {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;
    let local = world + half;

    if local.x < 0.0 || local.y < 0.0 {
        return None;
    }

    let x = (local.x / TILE_SIZE) as u32;
    let y = map
        .height
        .checked_sub(1 + (local.y / TILE_SIZE) as u32)
        .unwrap_or(u32::MAX);

    (x < map.width && y < map.height).then_some(TilePos::new(x, y))
}

pub fn look_image(index: usize) -> Image {
    Image::new(
        Extent3d {
            width: SIDE as u32,
            height: SIDE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        look_rgba(index),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD,
    )
}

fn forge(mut commands: Commands, images: Option<ResMut<Assets<Image>>>) {
    let Some(mut images) = images else { return };

    let handles = (0..LOOKS)
        .map(|index| images.add(look_image(index)))
        .collect();
    commands.insert_resource(CursorImages(handles));
}

fn track(
    map: Res<TileMap>,
    windows: Query<&Window>,
    camera: Option<Single<(&Camera, &GlobalTransform), With<WorldCamera>>>,
    mut hovered: ResMut<Hovered>,
) {
    let Some(camera) = camera else { return };
    let (camera, transform) = camera.into_inner();

    let found = windows
        .iter()
        .find_map(|window| window.cursor_position())
        .and_then(|cursor| camera.viewport_to_world_2d(transform, cursor).ok())
        .and_then(|world| tile_at(&map, world));

    hovered.pos = found;
    hovered.tile = found.and_then(|pos| map.get(pos));
}

fn pace(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut speed: ResMut<CursorSpeed>,
) {
    let held = buttons.is_some_and(|buttons| buttons.pressed(MouseButton::Left));
    let pointer = windows
        .single()
        .ok()
        .and_then(|window| window.physical_cursor_position());

    speed.step(held, pointer);
}

fn animate(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    over_ui: Option<Res<PointerOverUi>>,
    panning: Option<Res<Panning>>,
    hints: Res<CursorHints>,
    mut look: ResMut<CursorLook>,
) {
    let Some(buttons) = buttons else { return };
    let all = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
    let any_key = buttons.get_pressed().next().is_some()
        || keys.is_some_and(|keys| keys.get_pressed().next().is_some());

    let input = LookInput {
        down_any: buttons.any_just_pressed(all),
        down_right: buttons.just_pressed(MouseButton::Right),
        down_middle: buttons.just_pressed(MouseButton::Middle),
        up_any: buttons.any_just_released(all),
        up_right: buttons.just_released(MouseButton::Right),
        up_middle: buttons.just_released(MouseButton::Middle),
        held_right: buttons.pressed(MouseButton::Right),
        held_middle: buttons.pressed(MouseButton::Middle),
        any_key,
        over_ui: over_ui.is_some_and(|over| over.0),
        can_drag_map: panning.is_none_or(|panning| panning.can_drag_map),
        hints: *hints,
    };

    look.step(input);
}

fn wear(
    mut commands: Commands,
    look: Res<CursorLook>,
    images: Res<CursorImages>,
    windows: Query<(Entity, Option<&CursorIcon>), With<PrimaryWindow>>,
) {
    let Ok((window, worn)) = windows.single() else {
        return;
    };
    let Some(handle) = images.0.get(look.current as usize) else {
        return;
    };

    let wanted = CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
        handle: handle.clone(),
        hotspot: HOTSPOT,
        ..default()
    }));

    if worn != Some(&wanted) {
        commands.entity(window).insert(wanted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Tile};

    fn map() -> TileMap {
        TileMap::filled(448, 448, Tile::bare(Ground::DeepOcean))
    }

    #[test]
    fn the_centre_of_the_screen_is_the_middle_of_the_map() {
        let map = map();
        let pos = tile_at(&map, Vec2::ZERO).expect("inside");

        assert_eq!(pos, TilePos::new(224, 223));
    }

    #[test]
    fn the_top_left_tile_sits_at_the_top_left_corner() {
        let map = map();
        let half = 448.0 * TILE_SIZE * 0.5;
        let pos = tile_at(&map, Vec2::new(-half + 1.0, half - 1.0)).expect("inside");

        assert_eq!(pos, TilePos::new(0, 0));
    }

    #[test]
    fn the_bottom_right_tile_sits_at_the_bottom_right_corner() {
        let map = map();
        let half = 448.0 * TILE_SIZE * 0.5;
        let pos = tile_at(&map, Vec2::new(half - 1.0, -half + 1.0)).expect("inside");

        assert_eq!(pos, TilePos::new(447, 447));
    }

    #[test]
    fn pointing_off_the_map_finds_nothing() {
        let map = map();
        let half = 448.0 * TILE_SIZE * 0.5;

        assert!(tile_at(&map, Vec2::new(-half - 10.0, 0.0)).is_none());
        assert!(tile_at(&map, Vec2::new(0.0, half + 10.0)).is_none());
        assert!(tile_at(&map, Vec2::new(half + 10.0, 0.0)).is_none());
    }

    #[test]
    fn every_tile_can_be_pointed_at() {
        let map = TileMap::filled(16, 16, Tile::default());
        let half = 16.0 * TILE_SIZE * 0.5;

        for y in 0..16 {
            for x in 0..16 {
                let world = Vec2::new(
                    -half + (x as f32 + 0.5) * TILE_SIZE,
                    half - (y as f32 + 0.5) * TILE_SIZE,
                );

                assert_eq!(tile_at(&map, world), Some(TilePos::new(x, y)), "{x},{y}");
            }
        }
    }

    #[test]
    fn each_look_becomes_a_cursor_sized_image() {
        let image = look_image(DRAG as usize);

        assert_eq!(image.width(), SIDE as u32);
        assert_eq!(image.height(), SIDE as u32);
    }
}
