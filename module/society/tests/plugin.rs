use bevy::prelude::*;
use society::SocietyPlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(SocietyPlugin);
    app.update();
}
