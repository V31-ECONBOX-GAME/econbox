use bevy::prelude::*;
use population::PopulationPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(PopulationPlugin);
    app.update();
}
