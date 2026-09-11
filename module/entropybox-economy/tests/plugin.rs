use bevy::prelude::*;
use entropybox_economy::EconomyPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(EconomyPlugin);
    app.update();
}
