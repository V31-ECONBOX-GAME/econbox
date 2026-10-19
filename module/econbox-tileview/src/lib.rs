//! Draws the tile map the way WorldBox does: sprite tiles up close, one pixel per tile from afar.

pub mod art;
pub mod compose;
pub mod look;
pub mod quality;
pub mod view;

use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use econbox_camera::WorldBounds;
use econbox_tilemap::Painted;
use econbox_worldgen::{WorldConfig, generate};

pub use quality::{MapMode, Runup};
pub use view::{
    ChunkMaterial, Drawn, FLASH_LAYER, MAP_LAYER, MapLayer, TILES_LAYER, TileChunk, world_size,
};

pub const WORLD_LAYER: f32 = TILES_LAYER;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSeed(pub u32);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(WorldConfig::default().seed)
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WorldSettings(pub WorldConfig);

#[derive(Message, Debug, Clone, Copy)]
pub struct Regenerate {
    pub seed: u32,
}

pub struct TileViewPlugin;

impl Plugin for TileViewPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "chunk.wgsl");

        app.add_plugins(Material2dPlugin::<ChunkMaterial>::default())
            .init_resource::<WorldSeed>()
            .init_resource::<WorldSettings>()
            .init_resource::<MapMode>()
            .init_resource::<Runup>()
            .init_resource::<Drawn>()
            .add_message::<Regenerate>()
            .add_message::<Painted>()
            .add_systems(Startup, (spawn, view::spawn_layers))
            .add_systems(
                Update,
                (
                    rebuild,
                    view::refresh,
                    view::repaint,
                    view::switch,
                    view::pulse,
                    view::draw,
                )
                    .chain(),
            );
    }
}

fn spawn(
    mut commands: Commands,
    mut bounds: ResMut<WorldBounds>,
    settings: Res<WorldSettings>,
    seed: Res<WorldSeed>,
) {
    let map = generate(WorldConfig {
        seed: seed.0,
        ..settings.0
    });
    *bounds = WorldBounds::from_tiles(map.width, map.height);

    commands.insert_resource(map);
}

fn rebuild(
    mut commands: Commands,
    mut asked: MessageReader<Regenerate>,
    mut bounds: ResMut<WorldBounds>,
    mut seed: ResMut<WorldSeed>,
    settings: Res<WorldSettings>,
) {
    let Some(request) = asked.read().last().copied() else {
        return;
    };

    seed.0 = request.seed;
    let map = generate(WorldConfig {
        seed: request.seed,
        ..settings.0
    });

    *bounds = WorldBounds::from_tiles(map.width, map.height);

    commands.insert_resource(map);
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_camera::TILE_SIZE;
    use econbox_tile::Tile;
    use econbox_tilemap::TileMap;

    #[test]
    fn the_world_is_the_map_measured_in_tiles() {
        let map = TileMap::filled(448, 448, Tile::default());

        assert_eq!(world_size(&map), Vec2::splat(448.0 * TILE_SIZE));
    }

    #[test]
    fn the_map_layer_sits_above_the_tiles_and_both_below_the_creatures() {
        const {
            assert!(MAP_LAYER > TILES_LAYER);
            assert!(FLASH_LAYER > MAP_LAYER);
            assert!(FLASH_LAYER < 0.5);
        }
    }
}
