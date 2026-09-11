mod common;

use entropybox_world::WorldPlugin;

#[test]
fn assembly_registers_simulation_plugins() {
    let app = common::run(1);

    assert!(app.is_plugin_added::<WorldPlugin>());
}
