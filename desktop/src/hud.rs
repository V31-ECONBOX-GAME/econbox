//! The readout that follows the pointer, and the key that reshapes the world.

use crate::prelude::*;
use crate::world::{Land, Survey};

#[derive(Component)]
struct HudLine(usize);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Survey>()
            .add_systems(Startup, spawn)
            .add_systems(Update, (reshape, measure, refresh).chain());
    }
}

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(11.0),
                top: Val::Px(11.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.09, 0.74)),
        ))
        .with_children(|panel| {
            for line in 0..4 {
                panel.spawn((
                    HudLine(line),
                    Text::new(""),
                    TextFont::from_font_size(10.0),
                    TextColor(Color::srgb(0.88, 0.92, 0.96)),
                ));
            }
        });
}

fn reshape(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    seed: Res<WorldSeed>,
    mut asked: MessageWriter<Regenerate>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        asked.write(Regenerate {
            seed: next_seed(seed.0, time.elapsed_secs()),
        });
    }
}

fn next_seed(current: u32, seconds: f32) -> u32 {
    let mut seed = current ^ (seconds.to_bits().rotate_left(13));
    seed ^= seed >> 16;
    seed = seed.wrapping_mul(0x7feb_352d);
    seed ^= seed >> 15;
    seed.wrapping_mul(0x846c_a68b) | 1
}

fn measure(map: Option<Res<TileMap>>, mut survey: ResMut<Survey>) {
    let Some(map) = map else { return };

    if map.is_changed() {
        *survey = Survey::of(&map);
    }
}

fn refresh(
    map: Option<Res<TileMap>>,
    seed: Res<WorldSeed>,
    hovered: Res<Hovered>,
    view: Res<MapView>,
    survey: Res<Survey>,
    mut text: Query<(&HudLine, &mut Text)>,
) {
    let Some(map) = map else { return };

    let readout = lines(
        hovered.pos.and_then(|pos| Land::at(&map, pos)),
        &survey,
        seed.0,
        (map.width, map.height),
        view.visible_tiles,
    );

    for (line, mut target) in &mut text {
        let Some(value) = readout.get(line.0) else {
            continue;
        };

        if target.as_str() != value.as_str() {
            **target = value.clone();
        }
    }
}

fn lines(
    land: Option<Land>,
    survey: &Survey,
    seed: u32,
    size: (u32, u32),
    visible: f32,
) -> [String; 4] {
    let (width, height) = size;

    [
        land.map_or_else(
            || "open sea".to_string(),
            |land| format!("{}, {}   {}", land.pos.x, land.pos.y, land.tile.name()),
        ),
        land.map_or_else(
            || "-".to_string(),
            |land| {
                let (x, y) = land.chunk();

                format!(
                    "chunk {x}/{y}   capacity {:.2}{}",
                    land.carrying_capacity(),
                    if land.is_settleable() {
                        "   settleable"
                    } else {
                        ""
                    }
                )
            },
        ),
        format!("seed {seed}   {width}x{height}   {visible:.0} tiles across"),
        format!(
            "water {:.0}%   land {:.0}%   settleable {:.0}%",
            survey.water_share() * 100.0,
            survey.land_share() * 100.0,
            survey.settleable_share() * 100.0
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn survey() -> Survey {
        Survey {
            cells: 200_704,
            land: 64_000,
            settleable: 40_000,
        }
    }

    fn grassland() -> Land {
        Land {
            pos: TilePos::new(100, 70),
            tile: Tile::new(Ground::SoilLow, Overlay::Grass),
        }
    }

    #[test]
    fn a_hovered_tile_reads_out_its_place_and_its_name() {
        let readout = lines(Some(grassland()), &survey(), 42, (448, 448), 96.0);

        assert_eq!(readout[0], "100, 70   soil_low:grass_low");
        assert!(readout[1].starts_with("chunk 1/1   capacity "));
        assert!(readout[1].ends_with("   settleable"));
    }

    #[test]
    fn pointing_at_nothing_reads_open_sea() {
        let readout = lines(None, &survey(), 42, (448, 448), 96.0);

        assert_eq!(readout[0], "open sea");
        assert_eq!(readout[1], "-");
    }

    #[test]
    fn deep_water_is_named_but_carries_nobody() {
        let readout = lines(
            Some(Land {
                pos: TilePos::new(5, 5),
                tile: Tile::bare(Ground::DeepOcean),
            }),
            &survey(),
            42,
            (448, 448),
            96.0,
        );

        assert_eq!(readout[0], "5, 5   deep_ocean");
        assert_eq!(readout[1], "chunk 0/0   capacity 0.00");
    }

    #[derive(Resource, Default)]
    struct Measures(usize);

    fn count(survey: Res<Survey>, mut measures: ResMut<Measures>) {
        if survey.is_changed() {
            measures.0 += 1;
        }
    }

    #[test]
    fn the_map_is_only_surveyed_when_it_changes() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .init_resource::<Survey>()
            .init_resource::<Measures>()
            .insert_resource(TileMap::filled(64, 64, Tile::bare(Ground::DeepOcean)))
            .add_systems(Update, (measure, count).chain());

        app.update();

        assert_eq!(app.world().resource::<Survey>().cells, 4096);
        assert_eq!(app.world().resource::<Measures>().0, 1);

        app.update();
        app.update();

        assert_eq!(app.world().resource::<Measures>().0, 1);

        app.world_mut().resource_mut::<TileMap>().set(
            0,
            0,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        );
        app.update();

        assert_eq!(app.world().resource::<Measures>().0, 2);
        assert_eq!(app.world().resource::<Survey>().land, 1);
    }

    #[test]
    fn the_world_lines_report_the_seed_and_the_shares() {
        let readout = lines(None, &survey(), 42, (448, 448), 96.0);

        assert_eq!(readout[2], "seed 42   448x448   96 tiles across");
        assert_eq!(readout[3], "water 68%   land 32%   settleable 20%");
    }
}
