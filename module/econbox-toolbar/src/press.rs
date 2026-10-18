//! What a click on the bar does, and how each button shows its state.

use crate::art::Kit;
use crate::bar::{ArrowFace, Scroll, StripTab, page};
use crate::catalog::{Def, Plate, TabId, tab};
use crate::motion::{Curve, Pop, TILT, Tilt};
use crate::tips::{Hover, Tip};
use crate::{Chosen, Controls, KeyCursor, Notice, Shown};
use bevy::picking::events::{PointerClick, PointerPress};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use econbox_brush::Tool;
use econbox_clock::WorldClock;
use econbox_creature::Census;

#[derive(Component, Debug, Clone, Copy)]
pub struct Slot {
    pub def: &'static Def,
}

#[derive(Component)]
pub struct SlotIcon;

#[derive(Component)]
pub struct SlotFrame;

#[derive(Component)]
pub struct ClockBadge;

#[derive(Component)]
pub struct PauseIcon;

#[derive(Component)]
pub struct BackButton;

pub const PRESS_SCALE: f32 = 0.9;
pub const HOVER_SCALE: f32 = 1.1;
pub const POP_TIME: f32 = 0.1;

pub fn press_pop() -> Pop {
    Pop::new(Vec2::splat(PRESS_SCALE), POP_TIME, Curve::InOutBack)
}

pub fn hover_pop() -> Pop {
    Pop::new(Vec2::splat(HOVER_SCALE), POP_TIME, Curve::InBack)
}

pub fn on_press(press: On<PointerPress>, mut scroll: ResMut<Scroll>) {
    if press.button == PointerButton::Primary && scroll.dragged {
        scroll.dragged = false;
    }
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    slots: Query<&Slot>,
    strip: Query<&StripTab>,
    backs: Query<(), With<BackButton>>,
    arrows: Query<&ArrowFace>,
    mut shown: ResMut<Shown>,
    mut scroll: ResMut<Scroll>,
    mut controls: Controls,
    mut cursor: ResMut<KeyCursor>,
) {
    let entity = click.entity;
    let primary = click.button == PointerButton::Primary;

    if let Ok(slot) = slots.get(entity) {
        click.propagate(false);

        if primary && scroll.dragged || click.button == PointerButton::Middle {
            return;
        }

        commands.entity(entity).insert(press_pop());
        cursor.0 = None;
        controls.press(slot.def);
        return;
    }

    if !primary {
        return;
    }

    if let Ok(cell) = strip.get(entity) {
        click.propagate(false);
        commands.entity(entity).insert(hover_pop());

        if shown.0 == cell.0 {
            shown.0 = TabId::Main;
        } else {
            shown.0 = cell.0;
            let definition = tab(cell.0);
            controls.notices.write(Notice(definition.title.to_string()));
        }
        return;
    }

    if backs.get(entity).is_ok() {
        click.propagate(false);

        if scroll.dragged {
            return;
        }

        commands.entity(entity).insert(press_pop());
        shown.0 = TabId::Main;
        return;
    }

    if let Ok(face) = arrows.get(entity) {
        click.propagate(false);
        page(&mut scroll, face.0);
    }
}

pub fn dress(
    chosen: Res<Chosen>,
    cursor: Res<KeyCursor>,
    census: Res<Census>,
    clock: Res<WorldClock>,
    kit: Option<Res<Kit>>,
    mut slots: Query<(&Slot, &Children, &mut ImageNode)>,
    mut icons: Query<(&mut ImageNode, &mut Tilt, Has<PauseIcon>), (With<SlotIcon>, Without<Slot>)>,
    mut frames: Query<&mut Visibility, With<SlotFrame>>,
    mut badges: Query<&mut ImageNode, (With<ClockBadge>, Without<Slot>, Without<SlotIcon>)>,
) {
    let Some(kit) = kit else { return };
    let face = if clock.paused { &kit.play } else { &kit.pause };

    for (slot, children, mut background) in &mut slots {
        let def = slot.def;
        let selected = chosen.0 == Some(def.id);
        let marked = selected || cursor.0 == Some(def.id);
        let living = match def.tool() {
            Some(Tool::Spawn(species)) => Some(census.of(species)),
            _ => None,
        };

        let wanted_background = if def.plate != Plate::Plain {
            kit.plate(def.plate)
        } else if living.is_some_and(|count| count > 0) {
            &kit.present
        } else {
            &kit.button
        };

        if background.image != *wanted_background {
            background.image = wanted_background.clone();
        }

        let tint = if !def.ready() {
            Color::srgba(0.6, 0.6, 0.6, 0.9)
        } else if living == Some(0) {
            Color::srgba(0.75, 0.75, 0.75, 0.9)
        } else {
            Color::WHITE
        };

        for child in children.iter() {
            if let Ok((mut image, mut tilt, pause)) = icons.get_mut(child) {
                if image.color != tint {
                    image.color = tint;
                }

                if pause && image.image != *face {
                    image.image = face.clone();
                }

                let target = if selected { TILT } else { 0.0 };
                if tilt.target != target {
                    tilt.target = target;
                }
            }

            if let Ok(mut visible) = frames.get_mut(child) {
                let wanted = if marked {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                visible.set_if_neq(wanted);
            }
        }
    }

    let badge = &kit.badges[clock.rung.min(kit.badges.len() - 1)];
    for mut image in &mut badges {
        if image.image != *badge {
            image.image = badge.clone();
        }
    }
}

pub fn hover(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    scroll: Res<Scroll>,
    changed: Query<(Entity, &Hovered), Changed<Hovered>>,
    slots: Query<&Slot>,
    strip: Query<&StripTab>,
    backs: Query<(), With<BackButton>>,
    tips: Query<&Tip>,
    mut hover: ResMut<Hover>,
) {
    let blocked = buttons.get_pressed().next().is_some()
        || scroll.held.is_some_and(|held| held.dragging)
        || hover.wheel_quiet() > 0.0;

    for (entity, hovered) in &changed {
        if !hovered.get() {
            if hover.target == Some(entity) {
                hover.leave();
            }
            if hover
                .pending
                .as_ref()
                .is_some_and(|(pending, _)| *pending == entity)
            {
                hover.pending = None;
            }
            continue;
        }

        let (content, pops) = if let Ok(slot) = slots.get(entity) {
            (Some(Tip::of(slot.def)), true)
        } else if let Ok(cell) = strip.get(entity) {
            let definition = tab(cell.0);
            (Some(Tip::plain(definition.title, definition.about)), true)
        } else if backs.get(entity).is_ok() {
            (Some(Tip::plain("Back", "Return to the main tab.")), true)
        } else {
            (tips.get(entity).ok().cloned(), false)
        };

        let Some(content) = content else { continue };

        if blocked {
            hover.pending = Some((entity, content));
            continue;
        }

        if pops {
            commands.entity(entity).insert(hover_pop());
        }

        hover.enter(entity, content);
    }

    if !blocked && let Some((entity, content)) = hover.pending.take() {
        hover.enter(entity, content);
    }
}
