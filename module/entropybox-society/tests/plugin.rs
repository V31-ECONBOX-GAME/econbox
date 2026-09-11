use bevy::prelude::*;
use entropybox_society::SocietyPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(SocietyPlugin);
    app.update();
}
