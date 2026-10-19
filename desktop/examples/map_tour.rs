use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use econbox_camera::{TILE_SIZE, WorldCamera, Zoom};
use econbox_tile::{Flora, Ground, Overlay, Tile, Wall};
use econbox_tilemap::{Painted, TileMap, TilePos};
use econbox_tileview::{MapMode, look};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
enum Step {
    Wait(u32),
    Span(f32),
    Look(f32, f32),
    Paint,
    Shot(&'static str),
}

#[derive(Resource)]
struct Tour {
    steps: Vec<Step>,
    at: usize,
    frames: u32,
    out: PathBuf,
    spot: Option<Vec2>,
}

fn script() -> Vec<Step> {
    vec![
        Step::Wait(90),
        Step::Shot("1-map"),
        Step::Span(150.0),
        Step::Look(0.0, 0.0),
        Step::Wait(90),
        Step::Shot("2-near"),
        Step::Span(60.0),
        Step::Look(0.0, 0.0),
        Step::Wait(90),
        Step::Shot("3-close"),
        Step::Span(26.0),
        Step::Look(0.0, 0.0),
        Step::Wait(90),
        Step::Shot("4-closer"),
        Step::Paint,
        Step::Wait(30),
        Step::Shot("5-painted"),
        Step::Span(300.0),
        Step::Look(0.0, 0.0),
        Step::Wait(90),
        Step::Shot("6-painted-map"),
        Step::Span(460.0),
        Step::Look(0.0, 0.0),
        Step::Wait(90),
        Step::Shot("7-far"),
        Step::Wait(5),
    ]
}

fn busiest(map: &TileMap) -> Vec2 {
    let mut best = (0, map.width / 2, map.height / 2);

    for cy in (80..map.height.saturating_sub(80)).step_by(6) {
        for cx in (100..map.width.saturating_sub(100)).step_by(6) {
            let mut kinds = HashSet::new();
            let mut water = 0;

            for y in (cy - 12..cy + 12).step_by(2) {
                for x in (cx - 20..cx + 20).step_by(2) {
                    let tile = map.at(x, y);
                    kinds.insert(look::of(tile).id);
                    water += usize::from(tile.is_water());
                }
            }

            let score = kinds.len() * 4 + water.min(60) / 3;
            if score > best.0 {
                best = (score, cx, cy);
            }
        }
    }

    Vec2::new(best.1 as f32, best.2 as f32)
}

fn world_of(map: &TileMap, tile: Vec2) -> Vec2 {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;
    Vec2::new(
        -half.x + (tile.x + 0.5) * TILE_SIZE,
        half.y - (tile.y + 0.5) * TILE_SIZE,
    )
}

fn paint(map: &mut TileMap, centre: Vec2) -> Vec<TilePos> {
    let mut cells = Vec::new();
    let (cx, cy) = (centre.x as u32, centre.y as u32);

    for y in cy - 3..=cy + 3 {
        for x in cx - 7..=cx - 2 {
            map.set(x, y, Tile::bare(Ground::Mountains));
            cells.push(TilePos::new(x, y));
        }
        for x in cx - 1..=cx + 4 {
            let tile = Tile::new(Ground::SoilLow, Overlay::Grass)
                .with_flora(((x + y) % 3 == 0).then_some(Flora::Tree));
            map.set(x, y, tile);
            cells.push(TilePos::new(x, y));
        }
        map.set(
            cx + 5,
            y,
            Tile::bare(Ground::SoilLow).with_wall(Some(Wall::Order)),
        );
        cells.push(TilePos::new(cx + 5, y));
    }

    cells
}

fn drive(
    mut commands: Commands,
    mut tour: ResMut<Tour>,
    mut zoom: ResMut<Zoom>,
    mut camera: Query<&mut Transform, With<WorldCamera>>,
    mut painted: MessageWriter<Painted>,
    mut exit: MessageWriter<AppExit>,
    mut map: Option<ResMut<TileMap>>,
    mode: Res<MapMode>,
) {
    if tour.spot.is_none()
        && let Some(map) = map.as_deref()
    {
        tour.spot = Some(busiest(map));
    }

    let Some(step) = tour.steps.get(tour.at).copied() else {
        exit.write(AppExit::Success);
        return;
    };

    match step {
        Step::Wait(frames) => {
            tour.frames += 1;
            if tour.frames < frames {
                return;
            }
            tour.frames = 0;
        }
        Step::Span(tiles) => {
            zoom.size = tiles * 0.5;
            zoom.target = tiles * 0.5;
        }
        Step::Look(dx, dy) => {
            if let (Some(map), Some(spot)) = (map.as_deref(), tour.spot) {
                let at = world_of(map, spot + Vec2::new(dx, dy));
                for mut transform in &mut camera {
                    transform.translation.x = at.x;
                    transform.translation.y = at.y;
                }
            }
        }
        Step::Paint => {
            if let (Some(map), Some(spot)) = (map.as_deref_mut(), tour.spot) {
                painted.write(Painted::new(paint(map, spot)));
            }
        }
        Step::Shot(name) => {
            let path = tour.out.join(format!("{name}.png"));
            info!(
                "shot {} low_res={} factor={}",
                path.display(),
                mode.low_res,
                mode.factor
            );
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
    }

    tour.at += 1;
}

fn main() {
    let out = std::env::var("ECONBOX_TOUR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("map-tour"));
    std::fs::create_dir_all(&out).expect("make tour folder");

    let mut app = desktop::app();
    app.insert_resource(Tour {
        steps: script(),
        at: 0,
        frames: 0,
        out,
        spot: None,
    })
    .add_systems(Update, drive);
    app.run();
}
