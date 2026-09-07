use bevy::prelude::*;
use politics::PoliticsPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(PoliticsPlugin);
    app.update();
}
