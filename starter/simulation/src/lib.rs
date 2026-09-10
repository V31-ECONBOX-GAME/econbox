use bevy::app::{PluginGroup, PluginGroupBuilder};
use entropybox_economy::EconomyPlugin;
use entropybox_finance::FinancePlugin;
use entropybox_politics::PoliticsPlugin;
use entropybox_population::PopulationPlugin;
use entropybox_society::SocietyPlugin;
use entropybox_world::WorldPlugin;

pub struct SimulationStarter;

impl PluginGroup for SimulationStarter {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(WorldPlugin)
            .add(PopulationPlugin)
            .add(SocietyPlugin)
            .add(EconomyPlugin)
            .add(FinancePlugin)
            .add(PoliticsPlugin)
    }
}
