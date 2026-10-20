mod common;

use econbox_worldgen::{WorldConfig, generate};

#[test]
fn a_generated_world_is_a_standard_continent_in_the_sea() {
    let map = generate(WorldConfig::with_seed(11));

    assert_eq!(map.chunks(), (4, 4));
    assert!(map.water_share() > 0.4 && map.water_share() < 0.9);
    assert!(map.land_cells() > 5_000);
}

#[test]
fn the_headless_app_ticks_without_a_window() {
    let app = common::run(8);

    assert!(app.world().entities().len() < 4096);
}
