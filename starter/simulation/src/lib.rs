use bevy::app::{PluginGroup, PluginGroupBuilder};
use economy::EconomyPlugin;
use finance::FinancePlugin;
use politics::PoliticsPlugin;
use population::PopulationPlugin;
use society::SocietyPlugin;
use world::WorldPlugin;

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
