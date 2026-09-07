use bevy::prelude::*;
use world::WorldPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(WorldPlugin);
    app.update();
}
