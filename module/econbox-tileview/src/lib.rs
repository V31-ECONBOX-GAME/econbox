//! Bakes the tile map into one pixel perfect texture and keeps it on screen.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use econbox_camera::{TILE_SIZE, WorldBounds};
use econbox_tilemap::{Painted, TileMap, TilePos};
use econbox_worldgen::{WorldConfig, generate};

pub const WORLD_LAYER: f32 = 0.0;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldSeed(pub u32);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(WorldConfig::default().seed)
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WorldSettings(pub WorldConfig);

#[derive(Resource, Debug, Clone)]
pub struct TileTexture(pub Handle<Image>);

#[derive(Component, Debug, Clone, Copy)]
pub struct WorldLayer;

#[derive(Message, Debug, Clone, Copy)]
pub struct Regenerate {
    pub seed: u32,
}

pub struct TileViewPlugin;

impl Plugin for TileViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldSeed>()
            .init_resource::<WorldSettings>()
            .add_message::<Regenerate>()
            .add_message::<Painted>()
            .add_systems(Startup, spawn)
            .add_systems(Update, (rebuild, repaint).chain());

        let settings = app.world().resource::<WorldSettings>().0;
        app.insert_resource(WorldBounds::from_tiles(settings.width(), settings.height()));
    }
}

pub fn bake(map: &TileMap) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: map.width,
            height: map.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        map.pixels(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });

    image
}

pub fn world_size(map: &TileMap) -> Vec2 {
    Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE
}

fn spawn(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut bounds: ResMut<WorldBounds>,
    settings: Res<WorldSettings>,
    seed: Res<WorldSeed>,
) {
    let map = generate(WorldConfig {
        seed: seed.0,
        ..settings.0
    });
    let texture = images.add(bake(&map));
    *bounds = WorldBounds::from_tiles(map.width, map.height);

    commands.spawn((
        WorldLayer,
        Sprite {
            image: texture.clone(),
            custom_size: Some(world_size(&map)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, WORLD_LAYER),
    ));

    commands.insert_resource(TileTexture(texture));
    commands.insert_resource(map);
}

fn rebuild(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut asked: MessageReader<Regenerate>,
    mut bounds: ResMut<WorldBounds>,
    mut seed: ResMut<WorldSeed>,
    mut sprites: Query<&mut Sprite, With<WorldLayer>>,
    settings: Res<WorldSettings>,
    texture: Option<Res<TileTexture>>,
) {
    let Some(request) = asked.read().last().copied() else {
        return;
    };
    let Some(texture) = texture else { return };

    seed.0 = request.seed;
    let map = generate(WorldConfig {
        seed: request.seed,
        ..settings.0
    });

    if let Some(mut image) = images.get_mut(&texture.0) {
        *image = bake(&map);
    }

    *bounds = WorldBounds::from_tiles(map.width, map.height);
    for mut sprite in &mut sprites {
        sprite.custom_size = Some(world_size(&map));
    }

    commands.insert_resource(map);
}

fn repaint(
    mut asked: MessageReader<Painted>,
    mut images: ResMut<Assets<Image>>,
    map: Option<Res<TileMap>>,
    texture: Option<Res<TileTexture>>,
) {
    let cells: Vec<TilePos> = asked
        .read()
        .flat_map(|painted| painted.cells.iter().copied())
        .collect();

    if cells.is_empty() {
        return;
    }

    let (Some(map), Some(texture)) = (map, texture) else {
        return;
    };
    let Some(mut image) = images.get_mut(&texture.0) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };

    for pos in cells {
        if pos.x >= map.width || pos.y >= map.height {
            continue;
        }

        let at = (pos.y * map.width + pos.x) as usize * 4;

        if at + 4 <= data.len() {
            data[at..at + 4].copy_from_slice(&map.pixel(pos.x, pos.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Tile};

    #[test]
    fn the_texture_is_one_pixel_per_tile() {
        let map = TileMap::filled(64, 48, Tile::bare(Ground::Sand));
        let image = bake(&map);

        assert_eq!(image.width(), 64);
        assert_eq!(image.height(), 48);
    }

    #[test]
    fn the_texture_carries_the_worldbox_colours() {
        let map = TileMap::filled(2, 1, Tile::bare(Ground::DeepOcean));
        let image = bake(&map);
        let data = image.data.expect("pixels");
        let base = Tile::bare(Ground::DeepOcean).color();

        for channel in 0..3 {
            let drift = i32::from(data[channel]) - i32::from(base[channel]);
            assert!(drift.abs() <= 8, "{drift}");
        }
    }

    #[test]
    fn a_repainted_cell_lands_at_its_own_offset_and_leaves_the_rest_alone() {
        let mut map = TileMap::filled(4, 3, Tile::bare(Ground::Sand));
        let mut data = bake(&map).data.expect("pixels");
        let untouched = map.pixel(0, 0);

        map.set(2, 1, Tile::bare(Ground::Mountains));
        let at = (map.width + 2) as usize * 4;
        data[at..at + 4].copy_from_slice(&map.pixel(2, 1));

        assert_eq!(data[at..at + 4], map.pixel(2, 1));
        assert_eq!(data[0..4], untouched);
        assert_ne!(data[at..at + 4], untouched);
    }

    #[test]
    fn the_world_bounds_follow_the_configured_map_size_before_startup() {
        let mut app = App::new();
        app.insert_resource(WorldBounds::default())
            .add_plugins(TileViewPlugin);

        let bounds = app.world().resource::<WorldBounds>();

        assert_eq!(bounds.width, 256.0 * TILE_SIZE);
        assert_eq!(bounds.height, 256.0 * TILE_SIZE);
    }

    #[test]
    fn the_sprite_is_the_map_measured_in_tiles() {
        let map = TileMap::filled(448, 448, Tile::default());

        assert_eq!(world_size(&map), Vec2::splat(448.0 * TILE_SIZE));
    }
}
