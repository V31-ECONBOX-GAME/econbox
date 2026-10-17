//! Keeps the bar looking like what the brush and the clock are actually doing.

use super::chrome::{Chrome, action_slot, extra_slot};
use super::{
    Action, ActionIcon, ActionSlot, ActiveTab, CLEAR, EDGE_LIT, EDGE_ON, GroupBox, Status, TabSlot,
    ToolSlot,
};
use crate::prelude::*;

pub(super) fn swap(
    brush: Res<Brush>,
    clock: Res<WorldClock>,
    mut icons: Query<(&ActionIcon, &mut ImageNode)>,
) {
    for (icon, mut node) in &mut icons {
        let index = match icon.0 {
            Action::Play if clock.paused => extra_slot(Icon::Play),
            Action::Shape if brush.shape == Shape::Square => extra_slot(Icon::Square),
            Action::Flood if brush.flood => extra_slot(Icon::Droplet),
            other => action_slot(other),
        };

        if let Some(atlas) = &mut node.texture_atlas
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}

pub(super) fn show(tab: Res<ActiveTab>, mut groups: Query<(&GroupBox, &mut Node)>) {
    if !tab.is_changed() {
        return;
    }

    for (group, mut node) in &mut groups {
        node.display = if group.0 == tab.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn edge(active: bool, lit: bool) -> Color {
    match (active, lit) {
        (true, _) => EDGE_ON,
        (false, false) => CLEAR,
        (false, true) => EDGE_LIT,
    }
}

pub(super) fn mark_tools(
    brush: Res<Brush>,
    mut slots: Query<(&ToolSlot, &PointerHovered, Has<Pressed>, &mut BorderColor)>,
) {
    for (slot, hovered, pressed, mut border) in &mut slots {
        let lit = hovered.get() || pressed;
        border.set_if_neq(BorderColor::all(edge(slot.0 == brush.tool, lit)));
    }
}

pub(super) fn mark_tabs(
    tab: Res<ActiveTab>,
    chrome: Option<Res<Chrome>>,
    mut slots: Query<(&TabSlot, &mut ImageNode), With<Button>>,
) {
    let Some(chrome) = chrome else { return };

    for (slot, mut node) in &mut slots {
        let wanted = if slot.0 == tab.0 {
            &chrome.tab_on
        } else {
            &chrome.tab
        };

        if node.image != *wanted {
            node.image = wanted.clone();
        }
    }
}

pub(super) fn mark_actions(
    brush: Res<Brush>,
    clock: Res<WorldClock>,
    mut slots: Query<(&ActionSlot, &PointerHovered, Has<Pressed>, &mut BorderColor)>,
) {
    for (slot, hovered, pressed, mut border) in &mut slots {
        let active = match slot.0 {
            Action::Flood => brush.flood,
            Action::Shape => brush.shape == Shape::Square,
            Action::Play => clock.paused,
            _ => false,
        };

        border.set_if_neq(BorderColor::all(edge(active, hovered.get() || pressed)));
    }
}

pub(super) fn status(
    brush: Res<Brush>,
    clock: Res<WorldClock>,
    undo: Res<UndoStack>,
    census: Res<Census>,
    hovered: Res<Hovered>,
    tab: Res<ActiveTab>,
    line: Option<Single<&mut Text, With<Status>>>,
) {
    let Some(line) = line else { return };

    let next = format!(
        "{}   tool {}   brush {}   clock {}   undo {}   alive {}   {}",
        tab.0.name(),
        brush.tool.name(),
        brush.label(),
        clock.label(),
        undo.depth(),
        census.alive,
        hovered.pos.map_or_else(
            || "off map".to_string(),
            |pos| format!("{}, {}", pos.x, pos.y)
        )
    );

    let mut line = line.into_inner();

    if line.as_str() != next {
        **line = next;
    }
}
