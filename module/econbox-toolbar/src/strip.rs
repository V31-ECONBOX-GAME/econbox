//! The tab strip's order, and holding a tab to drag it to a new place in the strip.

use crate::bar::{StripTab, cursor_canvas, px};
use crate::catalog::{STRIP, TabId};
use crate::layout::{STRIP_BOTTOM, STRIP_CELL_HEIGHT, STRIP_CELL_WIDTH};
use crate::{OpenWindow, UiHidden};
use bevy::prelude::*;

pub const HOLD: f32 = 0.25;
pub const FOLLOW: f32 = 10.0;
pub const STRIP_WIDTH: f32 = STRIP_CELL_WIDTH * STRIP.len() as f32;
const SETTLED: f32 = 0.1;
const BUTTONS: [MouseButton; 3] = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripOrder(pub [TabId; 6]);

impl Default for StripOrder {
    fn default() -> Self {
        Self(STRIP)
    }
}

impl StripOrder {
    pub fn index(&self, id: TabId) -> Option<usize> {
        self.0.iter().position(|cell| *cell == id)
    }

    pub fn home(&self, id: TabId) -> f32 {
        self.index(id).unwrap_or(0) as f32 * STRIP_CELL_WIDTH
    }

    pub fn next(&self, shown: TabId, backwards: bool) -> TabId {
        let count = self.0.len();

        match (self.index(shown), backwards) {
            (None, false) => self.0[0],
            (None, true) => self.0[count - 1],
            (Some(at), false) => self.0[(at + 1) % count],
            (Some(at), true) => self.0[(at + count - 1) % count],
        }
    }

    pub fn place(&mut self, id: TabId, to: usize) -> bool {
        let Some(from) = self.index(id) else {
            return false;
        };
        let to = to.min(self.0.len() - 1);

        if from < to {
            self.0[from..=to].rotate_left(1);
        } else if from > to {
            self.0[to..=from].rotate_right(1);
        }

        from != to
    }

    pub fn under(&self, along: f32, height: f32) -> Option<TabId> {
        let inside = (0.0..STRIP_WIDTH).contains(&along)
            && (STRIP_BOTTOM..STRIP_BOTTOM + STRIP_CELL_HEIGHT).contains(&height);
        inside.then(|| self.0[slot_at(along)])
    }
}

pub fn slot_at(along: f32) -> usize {
    ((along / STRIP_CELL_WIDTH).max(0.0) as usize).min(STRIP.len() - 1)
}

pub fn strip_left(canvas_width: f32) -> f32 {
    (canvas_width - STRIP_WIDTH) * 0.5
}

pub fn lifted_left(along: f32) -> f32 {
    (along - STRIP_CELL_WIDTH * 0.5).clamp(0.0, STRIP_WIDTH - STRIP_CELL_WIDTH)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grip {
    pub tab: TabId,
    pub button: MouseButton,
    pub held: f32,
    pub lifted: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct StripDrag {
    pub grip: Option<Grip>,
    pub moved: bool,
}

impl StripDrag {
    pub fn lifted(&self) -> Option<TabId> {
        self.grip.filter(|grip| grip.lifted).map(|grip| grip.tab)
    }

    pub fn press(&mut self, tab: TabId, button: MouseButton) {
        self.moved = false;
        self.grip = Some(Grip {
            tab,
            button,
            held: 0.0,
            lifted: false,
        });
    }

    pub fn hold(&mut self, order: &mut StripOrder, along: f32, over: bool, dt: f32) -> bool {
        let Some(grip) = &mut self.grip else {
            return false;
        };

        if !grip.lifted {
            if !over {
                self.grip = None;
                return false;
            }

            grip.held += dt;

            if grip.held < HOLD {
                return false;
            }

            grip.lifted = true;
            self.moved = true;
        }

        let centre = lifted_left(along) + STRIP_CELL_WIDTH * 0.5;
        order.place(grip.tab, slot_at(centre))
    }

    pub fn release(&mut self) {
        self.grip = None;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub x: f32,
    pub on_top: bool,
}

impl Cell {
    pub fn at(x: f32) -> Self {
        Self { x, on_top: false }
    }

    pub fn follow(&mut self, home: f32, dt: f32) {
        if (home - self.x).abs() < SETTLED {
            self.x = home;
            self.on_top = false;
        } else {
            self.x += (home - self.x) * (dt * FOLLOW).min(1.0);
        }
    }
}

pub fn drag(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    open: Res<OpenWindow>,
    hidden: Res<UiHidden>,
    mut order: ResMut<StripOrder>,
    mut grip: ResMut<StripDrag>,
    mut cells: Query<(&StripTab, &mut Cell, &mut Node, &mut ZIndex)>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };

    let dt = time.delta_secs();
    let left = strip_left(window.width() / scale.0);
    let cursor = cursor_canvas(window, scale.0).map(|at| Vec2::new(at.x - left, at.y));
    let over = cursor.and_then(|at| order.under(at.x, at.y));
    let usable = open.0.is_none() && !hidden.0;
    let held = grip.grip.map(|grip| grip.button);

    if !usable || held.is_some_and(|button| !buttons.pressed(button)) {
        grip.release();
    } else if held.is_some() {
        if let Some(at) = cursor {
            let on_it = over == grip.grip.map(|grip| grip.tab);
            grip.hold(&mut order, at.x, on_it, dt);
        }
    } else if let Some(button) = BUTTONS
        .into_iter()
        .find(|button| buttons.just_pressed(*button))
    {
        grip.moved = false;

        if let Some(tab) = over {
            grip.press(tab, button);
        }
    }

    let lifted = grip.lifted();

    for (tab, mut cell, mut node, mut layer) in &mut cells {
        if lifted == Some(tab.0)
            && let Some(at) = cursor
        {
            cell.x = lifted_left(at.x);
            cell.on_top = true;
        } else {
            cell.follow(order.home(tab.0), dt);
        }

        let wanted = px(cell.x);

        if node.left != wanted {
            node.left = wanted;
        }

        let z = ZIndex(i32::from(cell.on_top));

        if *layer != z {
            *layer = z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    fn centre(slot: usize) -> f32 {
        (slot as f32 + 0.5) * STRIP_CELL_WIDTH
    }

    #[test]
    fn placing_a_tab_shifts_the_ones_between() {
        let mut order = StripOrder::default();

        assert!(order.place(TabId::Creation, 3));
        assert_eq!(
            order.0,
            [
                TabId::Noosphere,
                TabId::Units,
                TabId::Nature,
                TabId::Creation,
                TabId::Destruction,
                TabId::Other,
            ]
        );

        assert!(order.place(TabId::Other, 0));
        assert_eq!(order.0[0], TabId::Other);
        assert_eq!(order.0[4], TabId::Creation);

        assert!(!order.place(TabId::Other, 0));
        assert!(!order.place(TabId::Main, 2));
    }

    #[test]
    fn every_place_keeps_each_tab_exactly_once() {
        for moved in STRIP {
            for to in 0..STRIP.len() + 2 {
                let mut order = StripOrder::default();
                order.place(moved, to);

                for id in STRIP {
                    assert_eq!(order.0.iter().filter(|cell| **cell == id).count(), 1);
                }
                assert_eq!(order.index(moved), Some(to.min(STRIP.len() - 1)));
            }
        }
    }

    #[test]
    fn the_slot_under_a_point_is_clamped_to_the_strip() {
        assert_eq!(slot_at(-30.0), 0);
        assert_eq!(slot_at(centre(0)), 0);
        assert_eq!(slot_at(centre(2)), 2);
        assert_eq!(slot_at(STRIP_WIDTH + 30.0), STRIP.len() - 1);

        let order = StripOrder::default();
        let row = STRIP_BOTTOM + STRIP_CELL_HEIGHT * 0.5;
        assert_eq!(order.under(centre(1), row), Some(TabId::Noosphere));
        assert_eq!(order.under(centre(1), STRIP_BOTTOM - 1.0), None);
        assert_eq!(order.under(-1.0, row), None);
    }

    #[test]
    fn a_quick_press_never_lifts_the_tab() {
        let mut order = StripOrder::default();
        let mut drag = StripDrag::default();
        drag.press(TabId::Creation, MouseButton::Left);

        for _ in 0..5 {
            drag.hold(&mut order, centre(5), true, FRAME);
        }
        drag.release();

        assert!(!drag.moved);
        assert_eq!(order, StripOrder::default());
    }

    #[test]
    fn leaving_the_tab_before_the_hold_cancels_the_drag() {
        let mut order = StripOrder::default();
        let mut drag = StripDrag::default();
        drag.press(TabId::Creation, MouseButton::Left);
        drag.hold(&mut order, centre(0), true, FRAME);
        drag.hold(&mut order, centre(3), false, FRAME);

        for _ in 0..30 {
            drag.hold(&mut order, centre(3), false, FRAME);
        }

        assert_eq!(drag.grip, None);
        assert_eq!(order, StripOrder::default());
    }

    #[test]
    fn holding_then_dragging_moves_the_tab_and_swallows_the_click() {
        let mut order = StripOrder::default();
        let mut drag = StripDrag::default();
        drag.press(TabId::Creation, MouseButton::Left);

        for _ in 0..20 {
            drag.hold(&mut order, centre(0), true, FRAME);
        }
        assert_eq!(drag.lifted(), Some(TabId::Creation));
        assert_eq!(order, StripOrder::default());

        for slot in 1..=4 {
            drag.hold(&mut order, centre(slot), false, FRAME);
            assert_eq!(order.index(TabId::Creation), Some(slot));
        }

        drag.hold(&mut order, STRIP_WIDTH + 200.0, false, FRAME);
        drag.release();

        assert!(drag.moved);
        assert_eq!(order.0[5], TabId::Creation);
        assert_eq!(order.0[4], TabId::Other);
    }

    #[test]
    fn tab_and_shift_tab_follow_the_new_order() {
        let mut order = StripOrder::default();
        order.place(TabId::Other, 0);

        assert_eq!(order.next(TabId::Main, false), TabId::Other);
        assert_eq!(order.next(TabId::Other, false), TabId::Creation);
        assert_eq!(order.next(TabId::Other, true), TabId::Destruction);
        assert_eq!(order.next(TabId::Main, true), TabId::Destruction);
    }

    #[test]
    fn a_dropped_tab_glides_home_and_comes_off_the_top() {
        let mut cell = Cell {
            x: 120.0,
            on_top: true,
        };

        cell.follow(0.0, FRAME);
        assert!(cell.x < 120.0 && cell.x > 0.0);
        assert!(cell.on_top);

        for _ in 0..120 {
            cell.follow(0.0, FRAME);
        }

        assert_eq!(cell, Cell::at(0.0));
    }
}
