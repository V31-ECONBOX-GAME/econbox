use bevy::prelude::*;
use entropybox_starter_simulation::SimulationStarter;

#[test]
fn group_builds() {
    let mut app = App::new();
    app.add_plugins(SimulationStarter);
    app.update();
}
