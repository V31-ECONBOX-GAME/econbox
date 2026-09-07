use bevy::app::{PluginGroup, PluginGroupBuilder};
use econbox_economy::EconomyPlugin;
use econbox_finance::FinancePlugin;
use econbox_politics::PoliticsPlugin;
use econbox_population::PopulationPlugin;
use econbox_society::SocietyPlugin;
use econbox_world::WorldPlugin;

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
