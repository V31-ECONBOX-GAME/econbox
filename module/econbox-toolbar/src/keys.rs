//! The keyboard and the wheel: tabs, the button cursor, saved slots, and the modifier wheel shortcuts.

use crate::bar::{Scroll, WHEEL_STEP, bring_into_view, cursor_canvas, over_track_band, place_of};
use crate::catalog::{Item, TabId, find, home_of, tab};
use crate::layout::{TOP_ROW, arrange};
use crate::sound::Cue;
use crate::strip::StripOrder;
use crate::windows::{Exit, Toward};
use crate::{Controls, KeyCursor, Notice, Shown, Slots};
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use econbox_brush::{PRESETS, Undo};

const FIRST_REPEAT: f32 = 0.33;
const NEXT_REPEAT: f32 = 0.1;

const DIGITS: [[KeyCode; 2]; 10] = [
    [KeyCode::Digit1, KeyCode::Numpad1],
    [KeyCode::Digit2, KeyCode::Numpad2],
    [KeyCode::Digit3, KeyCode::Numpad3],
    [KeyCode::Digit4, KeyCode::Numpad4],
    [KeyCode::Digit5, KeyCode::Numpad5],
    [KeyCode::Digit6, KeyCode::Numpad6],
    [KeyCode::Digit7, KeyCode::Numpad7],
    [KeyCode::Digit8, KeyCode::Numpad8],
    [KeyCode::Digit9, KeyCode::Numpad9],
    [KeyCode::Digit0, KeyCode::Numpad0],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Left,
    Right,
    Up,
    Down,
}

pub fn ctrl(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ])
}

pub fn shift(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

pub fn alt(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
}

pub fn walk(id: TabId, from: Option<&str>, step: Step) -> Option<&'static str> {
    let items = tab(id).items;
    let (places, _) = arrange(items);
    let stops: Vec<(f32, bool, &'static str)> = places
        .iter()
        .filter_map(|place| match &items[place.index] {
            Item::Button(def) if def.selectable() || def.id == "pause" => {
                Some((place.x, place.top == TOP_ROW, def.id))
            }
            _ => None,
        })
        .collect();

    if stops.is_empty() {
        return None;
    }

    let Some(current) = from.and_then(|id| stops.iter().find(|stop| stop.2 == id)) else {
        return Some(stops[0].2);
    };

    let (x, top, _) = *current;

    match step {
        Step::Left | Step::Right => {
            let row: Vec<_> = stops.iter().filter(|stop| stop.1 == top).collect();
            let at = row.iter().position(|stop| stop.2 == current.2)?;
            let next = if step == Step::Right {
                (at + 1) % row.len()
            } else {
                (at + row.len() - 1) % row.len()
            };
            Some(row[next].2)
        }
        Step::Up | Step::Down => {
            let other = stops
                .iter()
                .filter(|stop| stop.1 != top)
                .min_by(|a, b| (a.0 - x).abs().total_cmp(&(b.0 - x).abs()));
            other.map(|stop| stop.2).or(Some(current.2))
        }
    }
}

fn step_from(keys: &ButtonInput<KeyCode>, pressed: bool) -> Option<Step> {
    let check = |codes: [KeyCode; 2]| {
        if pressed {
            keys.any_pressed(codes)
        } else {
            keys.any_just_pressed(codes)
        }
    };

    if check([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(Step::Left)
    } else if check([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(Step::Right)
    } else if check([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(Step::Up)
    } else if check([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(Step::Down)
    } else {
        None
    }
}

pub fn hotkeys(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut controls: Controls,
    mut shown: ResMut<Shown>,
    mut exit: ResMut<Exit>,
    mut slots: ResMut<Slots>,
    mut cursor: ResMut<KeyCursor>,
    mut scroll: ResMut<Scroll>,
    order: Res<StripOrder>,
    mut cues: MessageWriter<Cue>,
    mut undo: MessageWriter<Undo>,
    mut held: Local<f32>,
) {
    let ctrl = ctrl(&keys);

    if keys.just_pressed(KeyCode::Escape) {
        if controls.open.0.is_some() {
            exit.0 = Toward::Left;
            controls.open.0 = None;
        } else if controls.hidden.0 {
            controls.hidden.0 = false;
        } else if controls.chosen.0.is_some() {
            controls.deselect();
            cursor.0 = None;
        } else if shown.0 != TabId::Main {
            shown.0 = TabId::Main;
            cues.write(Cue::Slide);
        }
        return;
    }

    if controls.open.0.is_some() {
        return;
    }

    if keys.just_pressed(KeyCode::KeyH) && !ctrl {
        controls.hidden.0 = !controls.hidden.0;
    }

    if keys.just_pressed(KeyCode::Space) {
        controls.clock.toggle();
    }

    if keys.just_pressed(KeyCode::Tab) {
        let next = order.next(shown.0, shift(&keys));
        shown.0 = next;
        cues.write(Cue::Slide);
        let definition = tab(next);
        controls.notices.write(Notice(format!(
            "{}\n{}",
            definition.title, definition.about
        )));
    }

    if ctrl {
        let fresh = step_from(&keys, false);
        let repeat = step_from(&keys, true);

        let fire = match (fresh, repeat) {
            (Some(step), _) => {
                *held = NEXT_REPEAT - FIRST_REPEAT;
                Some(step)
            }
            (None, Some(step)) => {
                *held += time.delta_secs();
                if *held >= NEXT_REPEAT {
                    *held = 0.0;
                    Some(step)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(step) = fire
            && let Some(id) = walk(shown.0, cursor.0.or(controls.chosen.0), step)
            && let Some(def) = find(id)
        {
            controls.deselect();
            cursor.0 = Some(id);

            if def.id != "pause" {
                controls.select(def);
            }

            if let Some(place) = place_of(shown.0, id) {
                bring_into_view(&mut scroll, place.x, false);
            }
        }
    }

    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
        && let Some(def) = cursor.0.and_then(find)
    {
        match def.id {
            "clock" => controls.clock.cycle(),
            _ if def.selectable() => {
                if controls.chosen.0 != Some(def.id) {
                    controls.select(def);
                }
            }
            _ => controls.press(def),
        }
    }

    for (slot, key) in DIGITS.iter().enumerate() {
        if !keys.any_just_pressed(*key) {
            continue;
        }

        let number = (slot + 1) % 10;

        if ctrl {
            match controls.chosen.def() {
                Some(def) => {
                    slots.0[slot] = Some(def.id);
                    controls
                        .notices
                        .write(Notice(format!("Saved {} to slot {number}.", def.name)));
                }
                None => {
                    controls.notices.write(Notice(
                        "Select a power first, then save it to a slot.".to_string(),
                    ));
                }
            }
            continue;
        }

        match slots.0[slot].and_then(find) {
            Some(def) if controls.chosen.0 == Some(def.id) => {
                cursor.0 = None;
                controls.deselect();
            }
            Some(def) => {
                cursor.0 = None;
                let here = crate::catalog::buttons(shown.0).any(|other| other.id == def.id);

                if !here && let Some(home) = home_of(def.id) {
                    shown.0 = home;
                    cues.write(Cue::Slide);
                }

                scroll.centre_on = Some(def.id);
                controls.select(def);
            }
            None => {
                controls.notices.write(Notice(format!(
                    "Slot {number} is empty. Hold Ctrl and press {number} to keep the selected power there."
                )));
            }
        }
    }

    if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace]) {
        let eraser = if controls.chosen.0 == Some("life_eraser") {
            "demolish"
        } else {
            "life_eraser"
        };

        if let Some(def) = find(eraser) {
            controls.press(def);
        }
    }

    if keys.just_pressed(KeyCode::KeyZ) && !ctrl {
        undo.write(Undo);
    }

    if keys.just_pressed(KeyCode::KeyF) {
        controls.clock.cycle();
    }

    if keys.just_pressed(KeyCode::BracketRight) {
        controls.brush.step(1);
    }

    if keys.just_pressed(KeyCode::BracketLeft) {
        controls.brush.step(-1);
    }
}

pub fn across(wheel: &AccumulatedMouseScroll) -> f32 {
    let per_line = match wheel.unit {
        MouseScrollUnit::Line => 1.0,
        MouseScrollUnit::Pixel => 20.0,
    };

    if wheel.delta.x.abs() > wheel.delta.y.abs() {
        wheel.delta.x / per_line
    } else {
        -wheel.delta.y / per_line
    }
}

pub fn wheel_lines(wheel: &AccumulatedMouseScroll) -> f32 {
    match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y,
        MouseScrollUnit::Pixel => wheel.delta.y / 20.0,
    }
}

pub fn wheel(
    wheel: Res<AccumulatedMouseScroll>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    mut controls: Controls,
    mut scroll: ResMut<Scroll>,
    mut brush_rest: Local<f32>,
    mut speed_rest: Local<f32>,
) {
    if wheel.delta == Vec2::ZERO || controls.open.0.is_some() {
        return;
    }

    let lines = wheel_lines(&wheel);

    if alt(&keys) {
        *brush_rest += lines;

        while brush_rest.abs() >= 1.0 {
            let by = brush_rest.signum() as i32;
            *brush_rest -= by as f32;

            if controls.brush.step(by) {
                let at = controls.brush.preset().unwrap_or(0);
                let preset = PRESETS[at];
                let name = if preset.flood {
                    "Fill"
                } else {
                    preset.group.name()
                };
                controls.notices.write(Notice(format!(
                    "Brush changed to: {name} ({})",
                    preset.radius
                )));
            }
        }
        return;
    }

    if ctrl(&keys) {
        *speed_rest += lines;

        while speed_rest.abs() >= 1.0 {
            let faster = *speed_rest > 0.0;
            *speed_rest -= speed_rest.signum();
            let before = controls.clock.rung;

            if faster {
                controls.clock.faster();
            } else {
                controls.clock.slower();
            }

            if controls.clock.rung != before {
                let label = controls.clock.speed().label;
                controls
                    .notices
                    .write(Notice(format!("Speed changed to: {label}")));
            }
        }
        return;
    }

    if shift(&keys) || controls.hidden.0 {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };

    if cursor_canvas(window, scale.0).is_some_and(over_track_band) {
        scroll.nudge(across(&wheel) * WHEEL_STEP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_cycles_through_the_strip_and_wraps() {
        let order = StripOrder::default();

        assert_eq!(order.next(TabId::Main, false), TabId::Creation);
        assert_eq!(order.next(TabId::Main, true), TabId::Other);
        assert_eq!(order.next(TabId::Other, false), TabId::Creation);
        assert_eq!(order.next(TabId::Units, false), TabId::Nature);
        assert_eq!(order.next(TabId::Creation, true), TabId::Other);
    }

    #[test]
    fn the_button_cursor_walks_rows_and_wraps() {
        let first = walk(TabId::Creation, None, Step::Right).expect("a stop");
        assert_eq!(first, "pause");

        let below = walk(TabId::Creation, Some("pause"), Step::Down).expect("below");
        assert_eq!(below, "clock");

        let right = walk(TabId::Creation, Some("pause"), Step::Right).expect("right");
        assert_eq!(right, "tile_deep_ocean");

        let wrapped = walk(TabId::Creation, Some("pause"), Step::Left).expect("wrapped");
        assert_ne!(wrapped, "pause");
    }

    #[test]
    fn the_cursor_skips_buttons_that_do_nothing_yet() {
        let mut at = "pause";

        for _ in 0..200 {
            at = walk(TabId::Destruction, Some(at), Step::Right).expect("a stop");
            let def = find(at).expect("a def");
            assert!(def.selectable() || def.id == "pause", "{at}");
        }
    }
}
