mod common;

use entropybox_economy::EconomyPlugin;
use entropybox_finance::FinancePlugin;
use entropybox_politics::PoliticsPlugin;
use entropybox_population::PopulationPlugin;
use entropybox_society::SocietyPlugin;
use entropybox_world::WorldPlugin;

#[test]
fn shipped_assembly_registers_every_simulation_plugin() {
    let app = common::run(1);

    assert!(app.is_plugin_added::<WorldPlugin>());
    assert!(app.is_plugin_added::<PopulationPlugin>());
    assert!(app.is_plugin_added::<SocietyPlugin>());
    assert!(app.is_plugin_added::<EconomyPlugin>());
    assert!(app.is_plugin_added::<FinancePlugin>());
    assert!(app.is_plugin_added::<PoliticsPlugin>());
}

#[test]
#[ignore]
fn long_run_does_not_panic() {
    let _ = common::run(10_000);
}
