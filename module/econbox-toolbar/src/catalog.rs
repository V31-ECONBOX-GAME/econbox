//! Every tab of the bar and every button on it, in the order they are laid out.

use crate::tabs::TABS;
use econbox_art::ui::TabMark;
use econbox_art::{Glyph, Icon, Mark, Stance};
use econbox_brush::Tool;
use econbox_creature::Species;
use econbox_tile::{Ground, Overlay};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Select,
    Spawn,
    Action,
    Toggle,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Act {
    Tool(Tool),
    Pause,
    Clock,
    HideUi,
    Planned(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tint {
    Keep,
    Rgb([u8; 3]),
    Ground(Ground),
    Overlay(Overlay),
}

impl Tint {
    pub fn color(self) -> Option<[u8; 3]> {
        match self {
            Self::Keep => None,
            Self::Rgb(color) => Some(color),
            Self::Ground(ground) => Some(ground.color()),
            Self::Overlay(overlay) => overlay.color(false),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Look {
    Icon(Icon, Tint),
    Glyph(Glyph, Option<[u8; 3]>, Option<[u8; 3]>),
    Creature(Species),
    Beast(Stance, Mark, [u8; 3], [u8; 3]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plate {
    Plain,
    Studded,
    Red,
    RedWell,
    Gilded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Def {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
    pub kind: Kind,
    pub act: Act,
    pub look: Look,
    pub sizes: bool,
    pub brush: Option<&'static str>,
    pub keep: bool,
    pub hint: Option<&'static str>,
    pub plate: Plate,
}

impl Def {
    pub fn ready(&self) -> bool {
        !matches!(self.act, Act::Planned(_))
    }

    pub fn selectable(&self) -> bool {
        self.ready() && matches!(self.kind, Kind::Select | Kind::Spawn)
    }

    pub fn tool(&self) -> Option<Tool> {
        match self.act {
            Act::Tool(tool) => Some(tool),
            _ => None,
        }
    }

    pub fn missing(&self) -> Option<&'static str> {
        match self.act {
            Act::Planned(system) => Some(system),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    AgeClock,
    PanelInfo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    Button(Def),
    Line,
    Thin,
    End,
    Space,
    Half,
    Back,
    Element(Element),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TabId {
    #[default]
    Main,
    Creation,
    Noosphere,
    Units,
    Destruction,
    Nature,
    Other,
}

pub const STRIP: [TabId; 6] = [
    TabId::Creation,
    TabId::Noosphere,
    TabId::Units,
    TabId::Nature,
    TabId::Destruction,
    TabId::Other,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabDef {
    pub id: TabId,
    pub title: &'static str,
    pub about: &'static str,
    pub mark: Option<TabMark>,
    pub items: &'static [Item],
}

pub fn tab(id: TabId) -> &'static TabDef {
    TABS.iter()
        .copied()
        .find(|tab| tab.id == id)
        .expect("every tab id has a definition")
}

pub fn buttons(id: TabId) -> impl Iterator<Item = &'static Def> {
    tab(id).items.iter().filter_map(|item| match item {
        Item::Button(def) => Some(def),
        _ => None,
    })
}

pub fn find(id: &str) -> Option<&'static Def> {
    TABS.iter()
        .flat_map(|tab| buttons(tab.id))
        .find(|def| def.id == id)
}

pub fn home_of(id: &str) -> Option<TabId> {
    TABS.iter()
        .find(|tab| tab.id != TabId::Main && buttons(tab.id).any(|def| def.id == id))
        .or_else(|| {
            TABS.iter()
                .find(|tab| buttons(tab.id).any(|def| def.id == id))
        })
        .map(|tab| tab.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn all() -> Vec<&'static Def> {
        TABS.iter().flat_map(|tab| buttons(tab.id)).collect()
    }

    #[test]
    fn every_tab_starts_with_pause_and_the_clock() {
        for tab in TABS {
            let ids: Vec<&str> = buttons(tab.id).take(2).map(|def| def.id).collect();

            assert_eq!(ids, ["pause", "clock"], "{:?}", tab.id);
        }
    }

    #[test]
    fn every_category_tab_has_a_back_button_and_main_has_none() {
        for tab in TABS {
            let has_back = tab.items.contains(&Item::Back);

            assert_eq!(has_back, tab.id != TabId::Main, "{:?}", tab.id);
        }
    }

    #[test]
    fn every_tab_ends_with_a_closing_line() {
        for tab in TABS {
            assert_eq!(tab.items.last(), Some(&Item::End), "{:?}", tab.id);
        }
    }

    #[test]
    fn button_ids_are_unique_apart_from_the_shared_time_controls() {
        let mut seen = HashSet::new();

        for def in all() {
            if def.id == "pause" || def.id == "clock" {
                continue;
            }

            assert!(seen.insert(def.id), "{} appears twice", def.id);
        }
    }

    #[test]
    fn the_bar_holds_every_button_of_the_reference_layout() {
        let count = |id: TabId| buttons(id).count();

        assert_eq!(count(TabId::Main), 15);
        assert_eq!(count(TabId::Creation), 49 + 6);
        assert_eq!(count(TabId::Noosphere), 65);
        assert_eq!(count(TabId::Units), 122 + 2);
        assert_eq!(count(TabId::Destruction), 39);
        assert_eq!(count(TabId::Nature), 55 + 2);
        assert_eq!(count(TabId::Other), 33);
    }

    #[test]
    fn every_ready_button_drives_something_real() {
        for def in all() {
            if def.ready() {
                assert!(
                    def.tool().is_some()
                        || matches!(def.act, Act::Pause | Act::Clock | Act::HideUi),
                    "{}",
                    def.id
                );
            } else {
                assert!(!def.missing().unwrap_or_default().is_empty(), "{}", def.id);
            }
        }
    }

    #[test]
    fn every_econbox_tool_has_a_button() {
        let tools: Vec<Tool> = all().iter().filter_map(|def| def.tool()).collect();

        for power in econbox_brush::POWERS {
            assert!(tools.contains(&Tool::Power(power)), "{}", power.name());
        }

        for species in econbox_creature::SPECIES {
            assert!(tools.contains(&Tool::Spawn(species)), "{}", species.name());
        }

        for ground in econbox_tile::GROUNDS {
            assert!(tools.contains(&Tool::Ground(ground)), "{}", ground.name());
        }

        for overlay in econbox_tile::OVERLAYS.iter().skip(1) {
            assert!(
                tools.contains(&Tool::Overlay(*overlay)),
                "{}",
                overlay.name()
            );
        }

        for bomb in econbox_brush::BOMBS {
            assert!(tools.contains(&Tool::Drop(bomb)), "{}", bomb.name());
            assert_eq!(
                find(bomb.name()).and_then(|def| def.tool()),
                Some(Tool::Drop(bomb))
            );
        }

        for wall in econbox_tile::WALLS {
            assert_eq!(
                find(wall.name()).and_then(|def| def.tool()),
                Some(Tool::Wall(wall))
            );
        }

        for ore in econbox_tile::ORES {
            assert!(tools.contains(&Tool::Ore(ore)), "{}", ore.name());
        }

        for tool in [
            Tool::Inspect,
            Tool::Raise,
            Tool::Lower,
            Tool::Strip,
            Tool::Plant(econbox_tile::Flora::Tree),
            Tool::Plant(econbox_tile::Flora::Bush),
            Tool::Fell,
            Tool::Mine,
        ] {
            assert!(tools.contains(&tool), "{tool:?}");
        }
    }

    #[test]
    fn every_spawn_button_shows_the_species_it_drops() {
        for def in all() {
            if let Some(Tool::Spawn(species)) = def.tool() {
                assert_eq!(def.kind, Kind::Spawn, "{}", def.id);
                assert_eq!(def.look, Look::Creature(species), "{}", def.id);
            }
        }
    }

    #[test]
    fn no_creature_button_waits_on_more_species() {
        for def in buttons(TabId::Units) {
            assert_ne!(def.missing(), Some("more species"), "{}", def.id);
        }
    }

    #[test]
    fn every_nature_seed_plant_and_deposit_is_live() {
        for def in buttons(TabId::Nature) {
            let id = def.id;

            if id.starts_with("seeds_") || id.starts_with("ore_") || id.starts_with("fertilizer_") {
                assert!(def.ready(), "{id}");
            }
        }

        for id in ["fruit_bush", "axe", "pickaxe", "sickle"] {
            assert!(find(id).is_some_and(Def::ready), "{id}");
        }
    }

    #[test]
    fn names_and_descriptions_are_written_out() {
        for def in all() {
            assert!(!def.name.is_empty(), "{}", def.id);
            assert!(def.about.ends_with('.'), "{}", def.id);
        }
    }

    #[test]
    fn a_button_is_found_in_its_category_before_main() {
        assert_eq!(home_of("tile_sand"), Some(TabId::Creation));
        assert_eq!(home_of("pause"), Some(TabId::Creation));
        assert_eq!(home_of("inspect"), Some(TabId::Main));
        assert_eq!(find("lightning").map(|def| def.name), Some("Lightning"));
    }

    #[test]
    fn forced_brushes_are_presets_the_brush_knows() {
        for def in all() {
            if let Some(id) = def.brush {
                assert!(
                    econbox_brush::PRESETS.iter().any(|preset| preset.id == id)
                        || ["sqr_0", "circ_1", "circ_10"].contains(&id),
                    "{} forces {id}",
                    def.id
                );
            }
        }
    }
}
