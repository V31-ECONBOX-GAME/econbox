//! The bottom panel: tabs above, tool plates below, and the hand on the clock.

mod chrome;
mod input;
mod layout;
mod paint;

use crate::prelude::*;

pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTab>()
            .add_observer(input::press)
            .add_systems(Startup, (chrome::forge, layout::spawn).chain())
            .add_systems(
                Update,
                (
                    layout::dress,
                    paint::swap,
                    input::over_ui,
                    input::shortcuts,
                    paint::show,
                    (
                        paint::mark_tools,
                        paint::mark_tabs,
                        paint::mark_actions,
                        paint::status,
                    ),
                )
                    .chain(),
            );
    }
}

const CLEAR: Color = Color::NONE;
const EDGE_LIT: Color = Color::srgb(0.72, 0.78, 0.62);
const EDGE_ON: Color = Color::srgb(0.98, 0.78, 0.26);
const DIM: Color = Color::srgb(0.62, 0.66, 0.56);

const SLOT_SIZE: f32 = 34.0;
const SLOT_GAP: f32 = 3.0;
const PIP: f32 = 24.0;
const ROWS: f32 = 2.0;
const STACK: f32 = SLOT_SIZE * ROWS + SLOT_GAP;

const TABS: [Tab; 4] = [Tab::Terrain, Tab::Nature, Tab::Sculpt, Tab::Creatures];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Terrain,
    Nature,
    Sculpt,
    Creatures,
}

impl Tab {
    fn name(self) -> &'static str {
        match self {
            Self::Terrain => "terrain",
            Self::Nature => "nature",
            Self::Sculpt => "sculpt",
            Self::Creatures => "creatures",
        }
    }

    fn badge(self) -> Icon {
        match self {
            Self::Terrain => Icon::Peaks,
            Self::Nature => Icon::Sprig,
            Self::Sculpt => Icon::Spade,
            Self::Creatures => Icon::Paw,
        }
    }

    fn groups(self) -> Vec<Vec<Tool>> {
        match self {
            Self::Terrain => vec![
                grounds(&[Ground::DeepOcean, Ground::CloseOcean, Ground::ShallowWaters]),
                grounds(&[Ground::Sand, Ground::SoilLow, Ground::SoilHigh]),
                grounds(&[Ground::Hills, Ground::Mountains]),
                grounds(&[Ground::Lava2, Ground::Lava3]),
            ],
            Self::Nature => vec![
                vec![
                    Tool::Overlay(Overlay::Grass),
                    Tool::Overlay(Overlay::Jungle),
                ],
                vec![
                    Tool::Overlay(Overlay::Enchanted),
                    Tool::Overlay(Overlay::Pumpkin),
                    Tool::Overlay(Overlay::Infernal),
                ],
                vec![Tool::Strip],
            ],
            Self::Sculpt => vec![vec![Tool::Raise, Tool::Lower], vec![Tool::Inspect]],
            Self::Creatures => [
                Kind::Folk,
                Kind::Farm,
                Kind::Beast,
                Kind::Aquatic,
                Kind::Monster,
            ]
            .iter()
            .map(|kind| {
                SPECIES
                    .iter()
                    .copied()
                    .filter(|species| species.kind() == *kind)
                    .map(Tool::Spawn)
                    .collect()
            })
            .collect(),
        }
    }

    fn tools(self) -> Vec<Tool> {
        self.groups().into_iter().flatten().collect()
    }
}

fn grounds(picked: &[Ground]) -> Vec<Tool> {
    picked.iter().copied().map(Tool::Ground).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Undo,
    Play,
    Speed,
    Shape,
    Shrink,
    Grow,
    Flood,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
struct ActiveTab(Tab);

#[derive(Component, Debug, Clone, Copy)]
struct ToolSlot(Tool);

#[derive(Component, Debug, Clone, Copy)]
struct TabSlot(Tab);

#[derive(Component, Debug, Clone, Copy)]
struct ActionSlot(Action);

#[derive(Component, Debug, Clone, Copy)]
struct SpeciesIcon(Species);

#[derive(Component, Debug, Clone, Copy)]
struct ToolIcon(Tool);

#[derive(Component, Debug, Clone, Copy)]
struct ActionIcon(Action);

#[derive(Component, Debug, Clone, Copy)]
struct GroupBox(Tab);

#[derive(Component)]
struct Bar;

#[derive(Component)]
struct TabStrip;

#[derive(Component)]
struct Status;
