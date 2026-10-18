//! The bar itself: its plating, the tab strip, one scrolling track per tab, and the edge arrows.

use crate::art::{Kit, bar_slices};
use crate::catalog::{Item, TabId, tab};
use crate::layout::*;
use crate::motion::{Curve, Mover, Pop, Tilt, in_out_circ, smooth_damp};
use crate::panels;
use crate::press::{BackButton, ClockBadge, PauseIcon, Slot, SlotFrame, SlotIcon};
use crate::strip::{Cell, STRIP_WIDTH, StripOrder};
use crate::tabs::TABS;
use crate::{KeyCursor, OpenWindow, Shown, UiHidden};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

pub const DRAG_START: f32 = 8.0;
pub const WHEEL_STEP: f32 = 20.0;
const ELASTICITY: f32 = 0.1;
const DECELERATION: f32 = 0.4;
const RELEASE_BOOST: f32 = 1.5;
const RUBBER: f32 = 0.55;

#[derive(Component)]
pub struct BarRoot;

#[derive(Component)]
pub struct Scroller;

#[derive(Component, Debug, Clone, Copy)]
pub struct Track(pub TabId);

#[derive(Component, Debug, Clone, Copy)]
pub struct StripTab(pub TabId);

#[derive(Component, Debug, Clone, Copy)]
pub struct Arrow {
    pub left: bool,
    pub timer: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ArrowFace(pub bool);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Held {
    start: f32,
    origin: f32,
    last: f32,
    pub dragging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glide {
    from: f32,
    to: f32,
    clock: f32,
    length: f32,
    back: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Scroll {
    pub offset: f32,
    pub velocity: f32,
    pub content: f32,
    pub view: f32,
    pub held: Option<Held>,
    pub dragged: bool,
    pub glide: Option<Glide>,
    pub centre_on: Option<&'static str>,
}

impl Scroll {
    pub fn floor(&self) -> f32 {
        (self.view - self.content).min(0.0)
    }

    pub fn overflow(&self) -> f32 {
        if self.offset > 0.0 {
            self.offset
        } else if self.offset < self.floor() {
            self.offset - self.floor()
        } else {
            0.0
        }
    }

    pub fn normalized(&self) -> f32 {
        let span = self.content - self.view;

        if span <= 0.0 {
            0.0
        } else {
            -self.offset / span
        }
    }

    pub fn offset_for(&self, normalized: f32) -> f32 {
        -normalized * (self.content - self.view).max(0.0)
    }

    pub fn reset(&mut self) {
        self.offset = 0.0;
        self.velocity = 0.0;
        self.glide = None;
        self.held = None;
    }

    pub fn glide_to(&mut self, to: f32, length: f32, back: bool) {
        self.velocity = 0.0;
        self.glide = Some(Glide {
            from: self.offset,
            to,
            clock: 0.0,
            length,
            back,
        });
    }

    pub fn nudge(&mut self, by: f32) {
        self.glide = None;
        self.offset += by;
    }

    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        if let Some(glide) = &mut self.glide {
            glide.clock += dt;
            let t = (glide.clock / glide.length).clamp(0.0, 1.0);
            let eased = if glide.back {
                crate::motion::out_back(t)
            } else {
                in_out_circ(t)
            };
            self.offset = glide.from + (glide.to - glide.from) * eased;

            if t >= 1.0 {
                self.glide = None;
            }

            return;
        }

        if self.held.is_some_and(|held| held.dragging) {
            return;
        }

        let over = self.overflow();

        if over != 0.0 {
            let mut speed = self.velocity;
            self.offset = smooth_damp(self.offset, self.offset - over, &mut speed, ELASTICITY, dt);
            self.velocity = if speed.abs() < 1.0 { 0.0 } else { speed };
        } else if self.velocity != 0.0 {
            self.velocity *= DECELERATION.powf(dt);

            if self.velocity.abs() < 1.0 {
                self.velocity = 0.0;
            }

            self.offset += self.velocity * dt;
        }
    }

    pub fn rubber(&self, wanted: f32) -> f32 {
        let floor = self.floor();
        let view = self.view.max(1.0);
        let stretch =
            |over: f32| (1.0 - 1.0 / (over.abs() * RUBBER / view + 1.0)) * view * over.signum();

        if wanted > 0.0 {
            stretch(wanted)
        } else if wanted < floor {
            floor + stretch(wanted - floor)
        } else {
            wanted
        }
    }

    pub fn press(&mut self, cursor: f32) {
        self.glide = None;
        self.velocity = 0.0;
        self.dragged = false;
        self.held = Some(Held {
            start: cursor,
            origin: self.offset,
            last: self.offset,
            dragging: false,
        });
    }

    pub fn drag(&mut self, cursor: f32, threshold: f32, dt: f32) {
        let Some(mut held) = self.held else { return };
        let delta = cursor - held.start;

        if !held.dragging && delta.abs() > threshold {
            held.dragging = true;
            self.dragged = true;
        }

        if held.dragging {
            self.offset = self.rubber(held.origin + delta);

            if dt > 0.0 {
                let fresh = (self.offset - held.last) / dt;
                self.velocity += (fresh - self.velocity) * (dt * 10.0).min(1.0);
            }

            held.last = self.offset;
        }

        self.held = Some(held);
    }

    pub fn release(&mut self) {
        if self.held.take().is_some_and(|held| held.dragging) {
            self.velocity *= RELEASE_BOOST;
        }
    }
}

pub fn px(value: f32) -> Val {
    Val::Px(value)
}

fn button_node(place: &Place) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(place.x),
        bottom: px(place.bottom()),
        width: px(place.width),
        height: px(place.height),
        ..default()
    }
}

pub fn icon_node() -> Node {
    let inset = (BUTTON - ICON) * 0.5;

    Node {
        position_type: PositionType::Absolute,
        left: px(inset),
        top: px(inset),
        width: px(ICON),
        height: px(ICON),
        ..default()
    }
}

pub fn frame_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0.0),
        top: px(0.0),
        width: px(BUTTON),
        height: px(BUTTON),
        ..default()
    }
}

pub fn badge_node() -> Node {
    let side = 20.95;

    Node {
        position_type: PositionType::Absolute,
        left: px(BUTTON * 0.5 + 9.8 - side * 0.5),
        top: px(BUTTON * 0.5 + 9.7 - side * 0.5),
        width: px(side),
        height: px(side),
        ..default()
    }
}

pub fn spawn(
    mut commands: Commands,
    kit: Option<Res<Kit>>,
    order: Res<StripOrder>,
    mut scroll: ResMut<Scroll>,
) {
    let Some(kit) = kit else { return };
    let shown = TabId::default();

    commands
        .spawn((
            BarRoot,
            Mover::new(Vec2::new(0.0, -300.0), true),
            UiTransform::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                bottom: px(0.0),
                height: px(0.0),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    right: px(0.0),
                    bottom: px(BAR_BOTTOM),
                    height: px(BAR_HEIGHT),
                    ..default()
                },
                ImageNode {
                    image: kit.bar.clone(),
                    image_mode: bar_slices(),
                    ..default()
                },
            ));

            bar.spawn((
                Scroller,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(CONTENT_LEFT),
                    bottom: px(TAB_ORIGIN),
                    width: px(0.0),
                    height: px(0.0),
                    ..default()
                },
            ))
            .with_children(|scroller| {
                for definition in TABS {
                    spawn_track(scroller, &kit, definition.id, definition.id == shown);
                }
            });

            for left in [true, false] {
                bar.spawn((
                    Arrow { left, timer: 1.0 },
                    Node {
                        position_type: PositionType::Absolute,
                        left: if left { px(0.0) } else { Val::Auto },
                        right: if left { Val::Auto } else { px(0.0) },
                        bottom: px(BAR_BOTTOM),
                        width: px(ARROW_WIDTH),
                        height: px(BAR_HEIGHT),
                        ..default()
                    },
                ))
                .with_children(|arrow| {
                    arrow.spawn((
                        ArrowFace(left),
                        Hovered::default(),
                        UiTransform::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0.0),
                            top: px(0.0),
                            width: px(ARROW_WIDTH),
                            height: px(BAR_HEIGHT),
                            ..default()
                        },
                        ImageNode {
                            image: kit.arrow.clone(),
                            flip_x: left,
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                    ));
                });
            }

            bar.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                margin: UiRect::left(px(-STRIP_WIDTH * 0.5)),
                bottom: px(STRIP_BOTTOM),
                width: px(STRIP_WIDTH),
                height: px(STRIP_CELL_HEIGHT),
                ..default()
            })
            .with_children(|strip| {
                for id in order.0 {
                    let home = order.home(id);

                    strip
                        .spawn((
                            StripTab(id),
                            Cell::at(home),
                            ZIndex(0),
                            Hovered::default(),
                            UiTransform::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(home),
                                top: px(0.0),
                                width: px(STRIP_CELL_WIDTH),
                                height: px(STRIP_CELL_HEIGHT),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ImageNode {
                                image: kit.tab.clone(),
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                        ))
                        .with_children(|cell| {
                            if let Some(mark) = kit.marks.get(&id) {
                                cell.spawn((
                                    Pickable::IGNORE,
                                    Node {
                                        width: px(33.0),
                                        height: px(12.0),
                                        margin: UiRect::top(px(2.0)),
                                        ..default()
                                    },
                                    ImageNode::new(mark.clone()),
                                ));
                            }
                        });
                }
            });
        });

    scroll.content = arrange(tab(shown).items).1;
}

fn spawn_track(parent: &mut ChildSpawnerCommands, kit: &Kit, id: TabId, shown: bool) {
    let definition = tab(id);
    let (places, _) = arrange(definition.items);

    parent
        .spawn((
            Track(id),
            UiTransform::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                bottom: px(0.0),
                width: px(0.0),
                height: px(0.0),
                display: if shown { Display::Flex } else { Display::None },
                ..default()
            },
        ))
        .with_children(|track| {
            for place in places {
                let item = &definition.items[place.index];

                match item {
                    Item::Button(def) => {
                        let mut button = track.spawn((
                            Slot { def },
                            Hovered::default(),
                            UiTransform::default(),
                            button_node(&place),
                            ImageNode {
                                image: kit.plate(def.plate).clone(),
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                        ));

                        button.with_children(|slot| {
                            let image = kit.icons.get(def.id).cloned().unwrap_or_default();
                            let mut icon = slot.spawn((
                                SlotIcon,
                                Tilt::default(),
                                Pickable::IGNORE,
                                UiTransform::default(),
                                icon_node(),
                                ImageNode::new(image),
                            ));

                            if def.id == "pause" {
                                icon.insert(PauseIcon);
                            }

                            if def.id == "clock" {
                                slot.spawn((
                                    ClockBadge,
                                    Pickable::IGNORE,
                                    badge_node(),
                                    ImageNode::new(kit.badges[1].clone()),
                                ));
                            }

                            slot.spawn((
                                SlotFrame,
                                Pickable::IGNORE,
                                Visibility::Hidden,
                                frame_node(),
                                ImageNode {
                                    image: kit.selection.clone(),
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                            ));
                        });
                    }
                    Item::Back => {
                        track
                            .spawn((
                                BackButton,
                                Hovered::default(),
                                UiTransform::default(),
                                button_node(&place),
                                ImageNode {
                                    image: kit.back_plate.clone(),
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                            ))
                            .with_children(|back| {
                                back.spawn((
                                    Pickable::IGNORE,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px((BUTTON - ICON) * 0.5),
                                        top: px((BACK_HEIGHT - ICON) * 0.5),
                                        width: px(ICON),
                                        height: px(ICON),
                                        ..default()
                                    },
                                    ImageNode::new(kit.back.clone()),
                                ));
                            });
                    }
                    Item::Line | Item::Thin | Item::End => {
                        let image = if line_width(item) == Some(THIN_LINE) {
                            kit.thin.clone()
                        } else {
                            kit.strong.clone()
                        };

                        track.spawn((
                            Pickable::IGNORE,
                            button_node(&place),
                            ImageNode {
                                image,
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                        ));
                    }
                    Item::Element(element) => {
                        panels::spawn_element(track, kit, *element, button_node(&place));
                    }
                    Item::Space | Item::Half => {}
                }
            }
        });
}

pub fn cursor_canvas(window: &Window, scale: f32) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    Some(Vec2::new(
        cursor.x / scale,
        (window.height() - cursor.y) / scale,
    ))
}

pub fn over_track_band(canvas: Vec2) -> bool {
    canvas.y >= -3.8 && canvas.y < STRIP_BOTTOM
}

pub fn drag(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    open: Res<OpenWindow>,
    hidden: Res<UiHidden>,
    mut scroll: ResMut<Scroll>,
    mut scroller: Query<&mut Node, With<Scroller>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };

    scroll.view = window.width() / scale.0 + 1.8;
    let usable = open.0.is_none() && !hidden.0;
    let cursor = cursor_canvas(window, scale.0);

    if buttons.just_pressed(MouseButton::Left)
        && usable
        && cursor.is_some_and(over_track_band)
        && let Some(at) = cursor
    {
        scroll.press(at.x);
    }

    if buttons.pressed(MouseButton::Left) {
        if let Some(at) = cursor {
            scroll.drag(at.x, DRAG_START / scale.0, time.delta_secs());
        }
    } else {
        scroll.release();
    }

    scroll.step(time.delta_secs());

    for mut node in &mut scroller {
        let wanted = px(CONTENT_LEFT + scroll.offset);

        if node.left != wanted {
            node.left = wanted;
        }
    }
}

pub fn switch(
    mut commands: Commands,
    shown: Res<Shown>,
    kit: Option<Res<Kit>>,
    mut scroll: ResMut<Scroll>,
    mut cursor: ResMut<KeyCursor>,
    mut tracks: Query<(Entity, &Track, &mut Node)>,
    mut strip: Query<(&StripTab, &mut ImageNode)>,
) {
    if !shown.is_changed() {
        return;
    }

    scroll.reset();
    scroll.content = arrange(tab(shown.0).items).1;
    cursor.0 = None;

    for (entity, track, mut node) in &mut tracks {
        let visible = track.0 == shown.0;
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };

        if visible && !shown.is_added() {
            commands
                .entity(entity)
                .insert(Pop::new(Vec2::new(0.2, 0.9), 0.2, Curve::OutCubic));
        }
    }

    if let Some(kit) = kit {
        for (cell, mut image) in &mut strip {
            let wanted = if cell.0 == shown.0 {
                &kit.tab_on
            } else {
                &kit.tab
            };

            if image.image != *wanted {
                image.image = wanted.clone();
            }
        }
    }
}

pub fn arrows(
    time: Res<Time>,
    shown: Res<Shown>,
    mut scroll: ResMut<Scroll>,
    mut arrows: Query<&mut Arrow>,
    mut faces: Query<(&ArrowFace, &mut UiTransform)>,
) {
    if let Some(id) = scroll.centre_on.take()
        && let Some(place) = place_of(shown.0, id)
    {
        bring_into_view(&mut scroll, place.x, true);
    }

    let fits = scroll.content < scroll.view;
    let at = scroll.normalized();

    for mut arrow in &mut arrows {
        let show = if fits {
            false
        } else if arrow.left {
            at > 0.1
        } else {
            at < 0.98 && at != 1.0
        };

        let step = time.delta_secs() * 2.0;
        arrow.timer = if show {
            (arrow.timer - step).max(0.0)
        } else {
            (arrow.timer + step).min(1.0)
        };

        let shift = ARROW_HIDE * in_out_circ(arrow.timer);
        let wanted = if arrow.left { -shift } else { shift };

        for (face, mut transform) in &mut faces {
            if face.0 == arrow.left {
                transform.translation = Val2::px(wanted, 0.0);
            }
        }
    }
}

pub fn page(scroll: &mut Scroll, left: bool) {
    if scroll.content <= 0.0 {
        return;
    }

    let share = (scroll.view / scroll.content).min(0.5);
    let target = if left {
        scroll.normalized() - share
    } else {
        scroll.normalized() + share
    };
    let to = scroll.offset_for(target);
    scroll.glide_to(to, 0.3, false);
}

pub fn bring_into_view(scroll: &mut Scroll, left: f32, centred: bool) {
    let to = if centred {
        let wanted = -(left + BUTTON * 0.5 - scroll.view * 0.5);
        wanted.clamp(scroll.floor(), 0.0)
    } else {
        let right_edge = left + BUTTON + scroll.offset;

        if right_edge > scroll.view - 96.0 + BUTTON {
            -(left + BUTTON - (scroll.view - 96.0 + BUTTON))
        } else if left + scroll.offset < 0.0 {
            -left + START
        } else {
            return;
        }
    };

    let to = to.clamp(scroll.floor().min(0.0), 0.0);
    let length = if centred { 0.125 } else { 0.25 };
    scroll.glide_to(to, length, centred);
}

pub fn place_of(id: TabId, button: &str) -> Option<Place> {
    let items = tab(id).items;
    let (places, _) = arrange(items);
    places
        .into_iter()
        .find(|place| matches!(items[place.index], Item::Button(def) if def.id == button))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scroll(content: f32, view: f32) -> Scroll {
        Scroll {
            content,
            view,
            ..default()
        }
    }

    #[test]
    fn a_short_tab_never_scrolls() {
        let mut short = scroll(300.0, 800.0);
        short.nudge(-200.0);

        for _ in 0..120 {
            short.step(1.0 / 60.0);
        }

        assert!(short.offset.abs() < 0.5, "{}", short.offset);
    }

    #[test]
    fn overscrolling_springs_back_to_the_last_item() {
        let mut long = scroll(2000.0, 800.0);
        long.nudge(-1500.0);

        for _ in 0..120 {
            long.step(1.0 / 60.0);
        }

        assert!((long.offset - -1200.0).abs() < 0.5, "{}", long.offset);
    }

    #[test]
    fn a_small_wiggle_is_still_a_click() {
        let mut long = scroll(2000.0, 800.0);
        long.press(100.0);
        long.drag(103.0, DRAG_START, 0.016);
        long.release();

        assert!(!long.dragged);
        assert_eq!(long.offset, 0.0);
    }

    #[test]
    fn a_real_drag_moves_the_bar_and_coasts_after_release() {
        let mut long = scroll(2000.0, 800.0);
        long.press(400.0);

        for step in 1..=10 {
            long.drag(400.0 - step as f32 * 20.0, DRAG_START, 1.0 / 60.0);
        }

        assert!(long.dragged);
        let held = long.offset;
        assert!(held < -150.0, "{held}");

        long.release();
        long.step(1.0 / 60.0);
        assert!(long.offset < held);
    }

    #[test]
    fn dragging_past_the_start_stretches_less_than_the_pointer() {
        let mut long = scroll(2000.0, 800.0);
        long.press(100.0);
        long.drag(300.0, DRAG_START, 0.016);

        assert!(long.offset > 0.0 && long.offset < 200.0, "{}", long.offset);
    }

    #[test]
    fn paging_right_moves_by_a_screen_or_half_the_content() {
        let mut long = scroll(4000.0, 800.0);
        page(&mut long, false);

        for _ in 0..40 {
            long.step(1.0 / 60.0);
        }

        let moved = -long.offset;
        assert!((moved - 800.0 / 4000.0 * 3200.0).abs() < 1.0, "{moved}");
    }
}
