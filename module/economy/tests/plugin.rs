use bevy::prelude::*;
use economy::EconomyPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);
    app.update();
}
