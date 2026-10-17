//! Clicks on the bar and the keys that do the same thing.

use super::{Action, ActionSlot, ActiveTab, Bar, TabSlot, ToolSlot};
use crate::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub(super) fn over_ui(
    bar: Option<Single<&RelativeCursorPosition, With<Bar>>>,
    mut over: ResMut<PointerOverUi>,
) {
    let inside = bar.is_some_and(|bar| bar.cursor_over());

    if over.0 != inside {
        over.0 = inside;
    }
}

pub(super) fn press(
    activate: On<Activate>,
    slots: Query<(Option<&ToolSlot>, Option<&TabSlot>, Option<&ActionSlot>)>,
    mut brush: ResMut<Brush>,
    mut tab: ResMut<ActiveTab>,
    mut clock: ResMut<WorldClock>,
    mut undo: MessageWriter<Undo>,
) {
    let Ok((tool, chosen, action)) = slots.get(activate.entity) else {
        return;
    };

    if let Some(tool) = tool {
        brush.tool = tool.0;
    }

    if let Some(chosen) = chosen {
        tab.0 = chosen.0;
    }

    if let Some(action) = action {
        match action.0 {
            Action::Undo => {
                undo.write(Undo);
            }
            Action::Play => clock.toggle(),
            Action::Speed => clock.faster(),
            Action::Shape => brush.shape = brush.shape.flipped(),
            Action::Shrink => brush.shrink(),
            Action::Grow => brush.grow(),
            Action::Flood => brush.flood = !brush.flood,
        }
    }
}

pub(super) fn shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut brush: ResMut<Brush>,
    mut clock: ResMut<WorldClock>,
    mut undo: MessageWriter<Undo>,
) {
    if keys.just_pressed(KeyCode::Space) {
        clock.toggle();
    }

    if keys.just_pressed(KeyCode::KeyF) {
        clock.faster();
    }

    if keys.just_pressed(KeyCode::KeyZ) {
        undo.write(Undo);
    }

    if keys.just_pressed(KeyCode::BracketLeft) {
        brush.shrink();
    }

    if keys.just_pressed(KeyCode::BracketRight) {
        brush.grow();
    }
}
