use crate::{Controls, Panel};
use bevy::prelude::*;

pub fn drops_power(panel: Panel) -> bool {
    panel != Panel::Brushes
}

pub fn on_window(mut controls: Controls, mut seen: Local<Option<Panel>>) {
    let open = controls.open.0;

    if open == *seen {
        return;
    }

    *seen = open;

    if open.is_some_and(drops_power) && controls.chosen.def().is_some_and(|def| !def.keep) {
        controls.deselect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Kind, TabId, buttons};

    #[test]
    fn only_the_brush_window_leaves_the_power_in_hand() {
        assert!(!drops_power(Panel::Brushes));
        assert!(drops_power(Panel::Other));
        assert!(drops_power(Panel::Settings));
        assert!(drops_power(Panel::Languages));
    }

    #[test]
    fn the_powers_worldbox_keeps_through_a_window_are_marked_kept() {
        let kept: Vec<&str> = [
            TabId::Main,
            TabId::Creation,
            TabId::Noosphere,
            TabId::Units,
            TabId::Destruction,
            TabId::Nature,
            TabId::Other,
        ]
        .into_iter()
        .flat_map(buttons)
        .filter(|def| def.keep && matches!(def.kind, Kind::Select | Kind::Spawn))
        .map(|def| def.id)
        .collect();

        for id in [
            "inspect",
            "city_select",
            "relations",
            "whisper_of_war",
            "unity",
        ] {
            assert!(kept.contains(&id), "{id}");
        }

        for id in ["cloud", "cloud_rain", "tile_sand", "lightning", "human"] {
            assert!(!kept.contains(&id), "{id}");
        }
    }
}
