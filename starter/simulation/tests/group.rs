use bevy::prelude::*;
use simulation::SimulationStarter;

#[test]
fn group_builds() {
    let mut app = App::new();
    app.add_plugins(SimulationStarter);
    app.update();
}
