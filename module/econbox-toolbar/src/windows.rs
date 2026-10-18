//! What every window shares: the frame, sliding in and out, the history behind the back button, scrolling, and the creature on its corner.

use crate::art::{Kit, sliced};
use crate::bar::{cursor_canvas, px};
use crate::motion::{out_back, out_cubic};
use crate::options::Options;
use crate::press::press_pop;
use crate::size::{UiLayer, layer_scale, window_size};
use crate::sound::Cue;
use crate::tips::{TITLE, Tip};
use crate::{OpenWindow, Panel};
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::picking::events::{PointerClick, PointerPress};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use econbox_art::Glyph;
use std::f32::consts::PI;

pub const ROOT_WIDTH: f32 = 222.0;
pub const ROOT_HEIGHT: f32 = 320.0;
pub const FRAME_WIDTH: f32 = 240.0;
pub const BODY_TOP: f32 = 21.0;
pub const HEADER_WIDTH: f32 = 140.0;
pub const HEADER_HEIGHT: f32 = 26.0;
pub const DROP: f32 = 6.0;
pub const SHOW_TIME: f32 = 0.35;
pub const HIDE_TIME: f32 = 0.1;
pub const SHOW_DELAY: f32 = 0.02;
pub const WHEEL_STEP: f32 = 60.0;
pub const VIEW_TOP: f32 = 160.0 + 6.5081 - 286.1854 * 0.5;
pub const VIEW_WIDTH: f32 = 232.0;
pub const VIEW_HEIGHT: f32 = 286.1854;
pub const CONTENT_WIDTH: f32 = 210.0;
pub const TRACK_LEFT: f32 = 120.0 + 124.72 - 17.444 * 0.5;
pub const TRACK_TOP: f32 = 160.0 + 33.091 - 112.0;
pub const TRACK_HEIGHT: f32 = 224.0;
pub const TABS_LEFT: f32 = -6.0 - 11.5;
pub const TABS_TOP: f32 = 200.7469 - 112.5;
pub const TAB_CELL: f32 = 22.0;
pub const FALL_TIME: f32 = 0.6;
pub const SPIN_TIME: f32 = 1.0;

pub fn shadow() -> TextShadow {
    TextShadow {
        offset: Vec2::new(0.7, 0.7),
        color: Color::srgba(0.0, 0.0, 0.0, 0.6),
    }
}

const SCROLLABLE: Color = Color::srgb(0.906, 0.325, 0.251);
const STUCK: Color = Color::srgb(0.329, 0.329, 0.329);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Toward {
    #[default]
    Right,
    Left,
}

impl Toward {
    fn sign(self) -> f32 {
        match self {
            Self::Right => 1.0,
            Self::Left => -1.0,
        }
    }
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct History(pub Vec<Panel>);

impl History {
    pub fn previous(&self) -> Option<Panel> {
        let n = self.0.len();
        (n >= 2).then(|| self.0[n - 2])
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Exit(pub Toward);

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shown(pub Panel);

#[derive(Component, Debug, Clone, Copy)]
pub struct WindowRoot {
    pub panel: Panel,
    pub width: f32,
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Slide {
    from: f32,
    to: f32,
    clock: f32,
    delay: f32,
    length: f32,
    bounce: bool,
    closing: bool,
}

impl Slide {
    pub fn enter(from: f32) -> Self {
        Self {
            from,
            to: 0.0,
            clock: 0.0,
            delay: SHOW_DELAY,
            length: SHOW_TIME,
            bounce: true,
            closing: false,
        }
    }

    pub fn leave(at: f32, to: f32) -> Self {
        Self {
            from: at,
            to,
            clock: 0.0,
            delay: SHOW_DELAY,
            length: HIDE_TIME,
            bounce: false,
            closing: true,
        }
    }

    pub fn x(&self) -> f32 {
        let t = ((self.clock - self.delay) / self.length.max(f32::EPSILON)).clamp(0.0, 1.0);
        let eased = if self.bounce { out_back(t) } else { t };
        self.from + (self.to - self.from) * eased
    }

    pub fn finished(&self) -> bool {
        self.clock >= self.delay + self.length
    }

    pub fn closing(&self) -> bool {
        self.closing
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CloseButton;

#[derive(Component, Debug, Clone, Copy)]
pub struct BackButton(pub Panel);

#[derive(Component, Debug, Clone, Copy)]
pub struct PreviousIcon(pub Panel);

#[derive(Component, Debug, Clone, Copy)]
pub struct Viewport(pub Panel);

#[derive(Component, Debug, Clone, Copy)]
pub struct Thumb(pub Panel);

#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct Critter {
    pub clock: Option<f32>,
    pub spin: f32,
    pub drop: f32,
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct ThumbDrag(pub Option<(Panel, f32, f32)>);

pub fn away(screen: Vec2, width: f32) -> f32 {
    screen.x * 0.5 + width * 0.5 + width * 0.2
}

pub fn screen_units(windows: &Query<&Window>, options: &Options) -> Vec2 {
    windows
        .iter()
        .next()
        .map(|window| {
            let size = window_size(window);
            size / layer_scale(UiLayer::Windows, size, options)
        })
        .unwrap_or(Vec2::new(ROOT_WIDTH * 3.0, ROOT_HEIGHT * 1.4))
}

pub fn route(
    last: Option<Panel>,
    next: Option<Panel>,
    history: &mut History,
    exit: Toward,
) -> (Option<(Panel, Toward)>, Option<(Panel, Toward)>) {
    match (last, next) {
        (None, Some(panel)) => {
            history.0 = vec![panel];
            (None, Some((panel, Toward::Right)))
        }
        (Some(from), Some(to)) if from == to => (None, None),
        (Some(from), Some(to)) => {
            if history.previous() == Some(to) {
                history.0.pop();
                (Some((from, Toward::Right)), Some((to, Toward::Left)))
            } else {
                history.0.push(to);
                (Some((from, Toward::Left)), Some((to, Toward::Right)))
            }
        }
        (Some(from), None) => {
            history.0.clear();
            (Some((from, exit)), None)
        }
        (None, None) => (None, None),
    }
}

pub fn navigate(
    open: Res<OpenWindow>,
    windows: Query<&Window>,
    options: Res<Options>,
    mut last: Local<Option<Panel>>,
    mut history: ResMut<History>,
    mut exit: ResMut<Exit>,
    mut roots: Query<(&WindowRoot, &mut Node, &mut Slide)>,
    mut critters: Query<(&mut Critter, &mut UiTransform, &mut Visibility)>,
    mut shown: MessageWriter<Shown>,
    mut cues: MessageWriter<Cue>,
) {
    if open.0 == *last {
        return;
    }

    let screen = screen_units(&windows, &options);
    let (going, coming) = route(*last, open.0, &mut history, exit.0);
    exit.0 = Toward::Right;
    *last = open.0;

    for (root, mut node, mut slide) in &mut roots {
        if let Some((panel, toward)) = going
            && root.panel == panel
        {
            *slide = Slide::leave(slide.x(), toward.sign() * away(screen, root.width));
        }

        if let Some((panel, toward)) = coming
            && root.panel == panel
        {
            node.display = Display::Flex;
            *slide = Slide::enter(toward.sign() * away(screen, root.width));
        }
    }

    if let Some((panel, _)) = coming {
        shown.write(Shown(panel));
        cues.write(Cue::Slide);

        for (mut critter, mut transform, mut visibility) in &mut critters {
            *critter = Critter::default();
            transform.translation = Val2::ZERO;
            transform.rotation = Rot2::IDENTITY;
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
}

pub fn slides(
    time: Res<Time>,
    mut roots: Query<(&mut Slide, &mut UiTransform, &mut Node), With<WindowRoot>>,
) {
    let dt = time.delta_secs();

    for (mut slide, mut transform, mut node) in &mut roots {
        if !slide.finished() {
            slide.clock += dt;
        }

        let wanted = Val2::px(slide.x(), DROP);
        if transform.translation != wanted {
            transform.translation = wanted;
        }

        if slide.closing && slide.finished() && node.display != Display::None {
            node.display = Display::None;
        }
    }
}

pub fn spawn_frame(
    commands: &mut Commands,
    kit: &Kit,
    panel: Panel,
    critter: Handle<Image>,
    critter_at: Vec2,
) -> (Entity, Entity) {
    let root = commands
        .spawn((
            WindowRoot {
                panel,
                width: FRAME_WIDTH,
            },
            Slide::default(),
            UiLayer::Windows,
            GlobalZIndex(600),
            UiTransform::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: px(-ROOT_WIDTH * 0.5),
                    top: px(-ROOT_HEIGHT * 0.5),
                    ..default()
                },
                width: px(ROOT_WIDTH),
                height: px(ROOT_HEIGHT),
                display: Display::None,
                ..default()
            },
        ))
        .id();

    let back_panel = panel;
    commands.entity(root).with_children(|window| {
        window
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(ROOT_WIDTH * 0.5 - 130.85 - 12.1437),
                    top: px(ROOT_HEIGHT * 0.5 - 116.6 - 13.5),
                    width: px(24.2874),
                    height: px(27.0),
                    display: Display::None,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackButton(back_panel),
                Hovered::default(),
                Tip::plain("Back", "Return to the previous window."),
                ImageNode {
                    image: kit.side_tab.clone(),
                    image_mode: sliced(6.0),
                    ..default()
                },
            ))
            .with_children(|back| {
                back.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(13.5 - 7.1784 - 14.0),
                        width: px(28.0),
                        height: px(28.0),
                        ..default()
                    },
                    ImageNode::new(kit.back.clone()),
                ));
            });

        window
            .spawn((
                PreviousIcon(back_panel),
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(ROOT_WIDTH * 0.5 - 126.96 - 8.25),
                    top: px(ROOT_HEIGHT * 0.5 - 98.3 - 9.0),
                    width: px(16.5),
                    height: px(18.0),
                    display: Display::None,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: kit.side_tab.clone(),
                    image_mode: sliced(6.0),
                    ..default()
                },
            ))
            .with_children(|icon| {
                icon.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(12.396),
                        height: px(14.1544),
                        ..default()
                    },
                    ImageNode::default(),
                ));
            });
    });

    let background = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: px((ROOT_WIDTH - FRAME_WIDTH) * 0.5),
            top: px(0.0),
            width: px(FRAME_WIDTH),
            height: px(ROOT_HEIGHT),
            ..default()
        })
        .id();
    commands.entity(root).add_child(background);

    commands.entity(background).with_children(|frame| {
        frame.spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(0.0),
                right: px(0.0),
                top: px(BODY_TOP),
                bottom: px(0.0),
                ..default()
            },
            ImageNode {
                image: kit.window.clone(),
                image_mode: sliced(12.0),
                ..default()
            },
        ));

        frame.spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px((FRAME_WIDTH - HEADER_WIDTH) * 0.5),
                top: px(0.0),
                width: px(HEADER_WIDTH),
                height: px(HEADER_HEIGHT),
                ..default()
            },
            ImageNode {
                image: kit.header.clone(),
                image_mode: sliced(8.0),
                ..default()
            },
        ));

        frame
            .spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    width: px(FRAME_WIDTH),
                    top: px(160.0 - 146.4 - 7.0),
                    height: px(14.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|title| {
                title.spawn((
                    Pickable::IGNORE,
                    Text::new(panel.title()),
                    TextFont::from_font_size(10.0),
                    TextColor(TITLE),
                    shadow(),
                ));
            });

        frame
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(FRAME_WIDTH * 0.5 + 104.2 - 20.0),
                    top: px(160.0 - 152.5 - 18.0),
                    width: px(40.0),
                    height: px(36.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode::new(kit.close_tab.clone()),
            ))
            .with_children(|tab| {
                tab.spawn((
                    CloseButton,
                    Hovered::default(),
                    Tip::plain("Close", "Close this window."),
                    UiTransform::default(),
                    Node {
                        width: px(32.0),
                        height: px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|close| {
                    close.spawn((
                        Pickable::IGNORE,
                        Node {
                            width: px(28.0),
                            height: px(28.0),
                            ..default()
                        },
                        ImageNode::new(kit.close.clone()),
                    ));
                });
            });

        frame.spawn((
            Critter::default(),
            Hovered::default(),
            UiTransform::default(),
            Visibility::Inherited,
            Node {
                position_type: PositionType::Absolute,
                left: px(critter_at.x - 14.0),
                top: px(critter_at.y - 14.0),
                width: px(28.0),
                height: px(28.0),
                ..default()
            },
            ImageNode::new(critter),
        ));
    });

    (root, background)
}

pub fn spawn_view(
    commands: &mut Commands,
    kit: &Kit,
    background: Entity,
    panel: Panel,
    spacing: f32,
    padding: (f32, f32),
    scrolls: bool,
) -> Entity {
    let mut content = Entity::PLACEHOLDER;

    commands.entity(background).with_children(|frame| {
        frame
            .spawn((
                Viewport(panel),
                ScrollPosition::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px((FRAME_WIDTH - VIEW_WIDTH) * 0.5),
                    top: px(VIEW_TOP),
                    width: px(VIEW_WIDTH),
                    height: px(VIEW_HEIGHT),
                    overflow: if scrolls {
                        Overflow::scroll_y()
                    } else {
                        Overflow::clip()
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|view| {
                content = view
                    .spawn(Node {
                        width: px(CONTENT_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(spacing),
                        padding: UiRect {
                            top: px(padding.0),
                            bottom: px(padding.1),
                            ..default()
                        },
                        flex_shrink: 0.0,
                        ..default()
                    })
                    .id();
            });

        if !scrolls {
            return;
        }

        frame
            .spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(TRACK_LEFT + 3.0),
                    top: px(TRACK_TOP),
                    width: px(11.5),
                    height: px(TRACK_HEIGHT),
                    ..default()
                },
                ImageNode {
                    image: kit.track.clone(),
                    image_mode: sliced(4.0),
                    ..default()
                },
            ))
            .with_children(|track| {
                track.spawn((
                    Thumb(panel),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(2.75),
                        top: px(0.0),
                        width: px(6.0),
                        height: px(TRACK_HEIGHT),
                        ..default()
                    },
                    ImageNode {
                        image: kit.handle.clone(),
                        image_mode: sliced(2.0),
                        color: STUCK,
                        ..default()
                    },
                ));
            });
    });

    content
}

pub fn spawn_tab_column(commands: &mut Commands, background: Entity) -> Entity {
    let mut column = Entity::PLACEHOLDER;
    commands.entity(background).with_children(|frame| {
        column = frame
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: px(TABS_LEFT),
                top: px(TABS_TOP),
                width: px(TAB_CELL),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .id();
    });
    column
}

pub fn scroll_limit(view: &ComputedNode) -> f32 {
    ((view.content_size().y - view.size().y) * view.inverse_scale_factor()).max(0.0)
}

pub fn wheel(
    open: Res<OpenWindow>,
    wheel: Res<AccumulatedMouseScroll>,
    mut views: Query<(&Viewport, &mut ScrollPosition, &ComputedNode)>,
) {
    let Some(panel) = open.0 else { return };

    if wheel.delta.y == 0.0 {
        return;
    }

    let step = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * WHEEL_STEP,
        MouseScrollUnit::Pixel => wheel.delta.y,
    };

    for (view, mut position, computed) in &mut views {
        if view.0 != panel {
            continue;
        }

        let limit = scroll_limit(computed);
        let next = (position.y - step).clamp(0.0, limit);
        if position.y != next {
            position.y = next;
        }
    }
}

pub fn thumbs(
    views: Query<(&Viewport, &ScrollPosition, &ComputedNode)>,
    mut thumbs: Query<(&Thumb, &mut Node, &mut ImageNode)>,
) {
    for (view, position, computed) in &views {
        let limit = scroll_limit(computed);
        let shown = computed.size().y * computed.inverse_scale_factor();
        let total = shown + limit;
        let height = if total > 0.0 {
            (TRACK_HEIGHT * shown / total).clamp(20.0, TRACK_HEIGHT)
        } else {
            TRACK_HEIGHT
        };
        let top = if limit > 0.0 {
            (TRACK_HEIGHT - height) * (position.y / limit).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let color = if limit > 0.5 { SCROLLABLE } else { STUCK };

        for (thumb, mut node, mut image) in &mut thumbs {
            if thumb.0 != view.0 {
                continue;
            }

            if node.height != px(height) || node.top != px(top) {
                node.height = px(height);
                node.top = px(top);
            }

            if image.color != color {
                image.color = color;
            }
        }
    }
}

pub fn drag_thumb(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    options: Res<Options>,
    mut drag: ResMut<ThumbDrag>,
    mut views: Query<(&Viewport, &mut ScrollPosition, &ComputedNode)>,
) {
    let Some((panel, start_y, start_scroll)) = drag.0 else {
        return;
    };

    if !buttons.pressed(MouseButton::Left) {
        drag.0 = None;
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };
    let scale = layer_scale(UiLayer::Windows, window_size(window), &options);
    let Some(cursor) = cursor_canvas(window, scale) else {
        return;
    };

    for (view, mut position, computed) in &mut views {
        if view.0 != panel {
            continue;
        }

        let limit = scroll_limit(computed);
        let shown = computed.size().y * computed.inverse_scale_factor();
        let height = (TRACK_HEIGHT * shown / (shown + limit).max(1.0)).clamp(20.0, TRACK_HEIGHT);
        let travel = (TRACK_HEIGHT - height).max(1.0);
        let moved = start_y - cursor.y;
        position.y = (start_scroll + moved * limit / travel).clamp(0.0, limit);
    }
}

pub fn back_buttons(
    history: Res<History>,
    kit: Option<Res<Kit>>,
    mut backs: Query<(&BackButton, &mut Node), Without<PreviousIcon>>,
    mut icons: Query<(&PreviousIcon, &mut Node, &Children), Without<BackButton>>,
    mut images: Query<&mut ImageNode>,
) {
    if !history.is_changed() {
        return;
    }

    let previous = history.previous();
    let display = if previous.is_some() {
        Display::Flex
    } else {
        Display::None
    };

    for (back, mut node) in &mut backs {
        let wanted = if history.0.last() == Some(&back.0) {
            display
        } else {
            node.display
        };
        if node.display != wanted {
            node.display = wanted;
        }
    }

    let Some(kit) = kit else { return };

    for (icon, mut node, children) in &mut icons {
        if history.0.last() != Some(&icon.0) {
            continue;
        }

        node.display = display;

        if let Some(previous) = previous
            && let Some(handle) = kit.glyphs.get(&previous.glyph())
        {
            for child in children.iter() {
                if let Ok(mut image) = images.get_mut(child) {
                    image.image = handle.clone();
                }
            }
        }
    }
}

pub fn critters(
    time: Res<Time>,
    windows: Query<&Window>,
    options: Res<Options>,
    mut critters: Query<(&mut Critter, &mut UiTransform, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    let fall = screen_units(&windows, &options).y;

    for (mut critter, mut transform, mut visibility) in &mut critters {
        let Some(clock) = critter.clock else { continue };
        let clock = clock + dt;
        critter.clock = Some(clock);

        let t = (clock / FALL_TIME).min(1.0);
        transform.translation = Val2::px(critter.drop, fall * t * t);
        let turn = out_cubic((clock / SPIN_TIME).min(1.0)) * critter.spin;
        transform.rotation = Rot2::radians(-turn * PI / 180.0);

        if clock >= FALL_TIME {
            visibility.set_if_neq(Visibility::Hidden);
        }
    }
}

fn wobble(seed: f32) -> f32 {
    (seed.sin() * 43_758.547).fract().abs()
}

pub fn on_press(
    mut press: On<PointerPress>,
    thumbs: Query<&Thumb>,
    views: Query<(&Viewport, &ScrollPosition)>,
    windows: Query<&Window>,
    options: Res<Options>,
    mut drag: ResMut<ThumbDrag>,
) {
    if press.button != PointerButton::Primary {
        return;
    }

    let Ok(thumb) = thumbs.get(press.entity) else {
        return;
    };
    press.propagate(false);

    let Some(window) = windows.iter().next() else {
        return;
    };
    let scale = layer_scale(UiLayer::Windows, window_size(window), &options);
    let Some(cursor) = cursor_canvas(window, scale) else {
        return;
    };

    let scroll = views
        .iter()
        .find(|(view, _)| view.0 == thumb.0)
        .map_or(0.0, |(_, position)| position.y);
    drag.0 = Some((thumb.0, cursor.y, scroll));
}

pub fn on_click(
    mut click: On<PointerClick>,
    mut commands: Commands,
    time: Res<Time>,
    closes: Query<(), With<CloseButton>>,
    backs: Query<(), With<BackButton>>,
    mut critters: Query<&mut Critter>,
    history: Res<History>,
    mut open: ResMut<OpenWindow>,
    mut exit: ResMut<Exit>,
    mut cues: MessageWriter<Cue>,
) {
    if click.button != PointerButton::Primary {
        return;
    }

    let entity = click.entity;

    if closes.contains(entity) {
        click.propagate(false);
        commands.entity(entity).insert(press_pop());
        cues.write(Cue::Click);
        exit.0 = Toward::Right;
        open.0 = None;
    } else if backs.contains(entity) {
        click.propagate(false);
        cues.write(Cue::Click);
        go_back(&history, &mut open, &mut exit);
    } else if let Ok(mut critter) = critters.get_mut(entity) {
        click.propagate(false);
        if critter.clock.is_none() {
            let seed = time.elapsed_secs();
            let spin = 90.0 + wobble(seed) * 90.0;
            critter.spin = if wobble(seed + 1.0) < 0.5 {
                spin
            } else {
                -spin
            };
            critter.drop = (wobble(seed + 2.0) - 0.5) * 8.0;
            critter.clock = Some(0.0);
        }
    }
}

pub fn go_back(history: &History, open: &mut OpenWindow, exit: &mut Exit) {
    match history.previous() {
        Some(previous) => open.0 = Some(previous),
        None => {
            exit.0 = Toward::Left;
            open.0 = None;
        }
    }
}

pub fn right_click(
    buttons: Res<ButtonInput<MouseButton>>,
    history: Res<History>,
    mut open: ResMut<OpenWindow>,
    mut exit: ResMut<Exit>,
) {
    if open.0.is_some() && buttons.just_pressed(MouseButton::Right) {
        go_back(&history, &mut open, &mut exit);
    }
}

impl Panel {
    pub fn title(self) -> &'static str {
        match self {
            Self::Brushes => "Brush",
            Self::Other => "Other",
            Self::Settings => "Settings",
        }
    }

    pub fn glyph(self) -> Glyph {
        match self {
            Self::Brushes => Glyph::Swirl,
            Self::Other | Self::Settings => Glyph::Gear,
        }
    }

    pub fn sidebar_button(self) -> Option<&'static str> {
        match self {
            Self::Brushes => None,
            Self::Other | Self::Settings => Some("settings_button"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_from_nothing_comes_in_from_the_right_and_starts_the_history() {
        let mut history = History::default();
        let (going, coming) = route(None, Some(Panel::Other), &mut history, Toward::Right);
        assert_eq!(going, None);
        assert_eq!(coming, Some((Panel::Other, Toward::Right)));
        assert_eq!(history.0, vec![Panel::Other]);
    }

    #[test]
    fn going_deeper_pushes_left_and_coming_back_returns_from_the_left() {
        let mut history = History(vec![Panel::Other]);
        let forward = route(
            Some(Panel::Other),
            Some(Panel::Settings),
            &mut history,
            Toward::Right,
        );
        assert_eq!(
            forward,
            (
                Some((Panel::Other, Toward::Left)),
                Some((Panel::Settings, Toward::Right))
            )
        );
        assert_eq!(history.previous(), Some(Panel::Other));

        let back = route(
            Some(Panel::Settings),
            Some(Panel::Other),
            &mut history,
            Toward::Right,
        );
        assert_eq!(
            back,
            (
                Some((Panel::Settings, Toward::Right)),
                Some((Panel::Other, Toward::Left))
            )
        );
        assert_eq!(history.0, vec![Panel::Other]);
    }

    #[test]
    fn closing_clears_the_history_and_leaves_the_way_it_was_told() {
        let mut history = History(vec![Panel::Other, Panel::Settings]);
        let (going, coming) = route(Some(Panel::Settings), None, &mut history, Toward::Left);
        assert_eq!(going, Some((Panel::Settings, Toward::Left)));
        assert_eq!(coming, None);
        assert!(history.0.is_empty());
    }

    #[test]
    fn a_window_bounces_in_and_slips_out_quickly() {
        let mut slide = Slide::enter(400.0);
        assert_eq!(slide.x(), 400.0);
        slide.clock = SHOW_DELAY + SHOW_TIME * 0.7;
        assert!(slide.x() < 0.0);
        slide.clock = SHOW_DELAY + SHOW_TIME;
        assert_eq!(slide.x(), 0.0);
        assert!(slide.finished());

        let mut out = Slide::leave(0.0, -400.0);
        out.clock = SHOW_DELAY + HIDE_TIME * 0.5;
        assert!((out.x() + 200.0).abs() < 1e-3);
        assert!(out.closing());
    }

    #[test]
    fn a_window_leaves_by_its_width_plus_a_fifth_beyond_the_screen_edge() {
        assert_eq!(away(Vec2::new(600.0, 420.0), 240.0), 300.0 + 120.0 + 48.0);
    }
}
