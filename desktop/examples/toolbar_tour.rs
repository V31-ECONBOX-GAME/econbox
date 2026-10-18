use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{CursorMoved, PrimaryWindow, WindowEvent};
use econbox_toolbar::catalog::{Item, TabId, tab};
use econbox_toolbar::layout::{
    STRIP_BOTTOM, STRIP_CELL_HEIGHT, STRIP_CELL_WIDTH, TAB_ORIGIN, arrange,
};
use econbox_toolbar::size::window_canvas_scale;
use econbox_toolbar::strip::strip_left;
use econbox_toolbar::{BarSize, StripOrder, canvas_scale};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum Step {
    Wait(u32),
    Point(Spot),
    Click(Spot),
    Press(Spot),
    Release,
    Drag(Spot, Spot),
    Key(KeyCode),
    Down(KeyCode),
    Up(KeyCode),
    Wheel(f32),
    Shot(&'static str),
}

#[derive(Debug, Clone, Copy)]
enum Spot {
    Canvas(f32, f32),
    Button(TabId, &'static str),
    Strip(TabId),
    Middle(f32),
    Right,
    Window(f32, f32),
}

#[derive(Resource)]
struct Tour {
    steps: Vec<Step>,
    at: usize,
    waiting: f32,
    out: PathBuf,
    drag: Option<(Vec2, Vec2, u32)>,
}

fn button_centre(id: TabId, button: &str) -> Vec2 {
    let items = tab(id).items;
    let (places, _) = arrange(items);
    let place = places
        .iter()
        .find(|place| matches!(items[place.index], Item::Button(def) if def.id == button))
        .expect("button on tab");
    let (x, y) = place.centre();
    Vec2::new(x, TAB_ORIGIN + y)
}

fn resolve(spot: Spot, size: Vec2, bar: BarSize, order: &StripOrder) -> Vec2 {
    if let Spot::Window(x, y) = spot {
        let scale = window_canvas_scale(size, 100);
        return Vec2::new(size.x * 0.5 + x * scale, size.y * 0.5 - y * scale);
    }

    let scale = canvas_scale(size, bar);
    let canvas = match spot {
        Spot::Canvas(x, y) => Vec2::new(x, y),
        Spot::Button(id, button) => button_centre(id, button),
        Spot::Strip(id) => {
            let at = order.index(id).expect("strip tab") as f32;
            Vec2::new(
                strip_left(size.x / scale) + STRIP_CELL_WIDTH * (at + 0.5),
                STRIP_BOTTOM + STRIP_CELL_HEIGHT * 0.5,
            )
        }
        Spot::Middle(y) => Vec2::new(size.x / scale * 0.5, y),
        Spot::Right => Vec2::new(size.x / scale - 5.0, 26.6),
        Spot::Window(..) => Vec2::ZERO,
    };

    Vec2::new(canvas.x * scale, size.y - canvas.y * scale)
}

fn settings_script() -> Vec<Step> {
    use Spot::*;
    use Step::*;

    let tab = |at: f32| Window(-126.5, 54.75 - 22.0 * at);
    let mut steps = vec![
        Wait(90),
        Point(Button(TabId::Main, "settings_button")),
        Wait(3),
        Click(Button(TabId::Main, "settings_button")),
        Wait(40),
        Shot("s01-other"),
        Point(Window(-60.0, -58.8)),
        Wait(20),
        Shot("s02-hover-settings-door"),
        Click(Window(-60.0, -58.8)),
        Wait(40),
        Shot("s03-settings-everything"),
        Point(Window(0.0, 0.0)),
        Wheel(-3.0),
        Wait(20),
        Shot("s04-scrolled"),
        Wheel(-30.0),
        Wait(20),
        Shot("s05-bottom"),
    ];

    for (at, name) in [
        (1.0, "s06-general"),
        (2.0, "s07-interface"),
        (3.0, "s08-graphics"),
        (4.0, "s09-other-tab"),
    ] {
        steps.extend([
            Point(tab(at)),
            Wait(3),
            Click(tab(at)),
            Wait(20),
            Shot(name),
        ]);
    }

    steps.extend([
        Point(tab(3.0)),
        Wait(3),
        Click(tab(3.0)),
        Wait(20),
        Point(Window(-78.0, -124.8)),
        Wait(3),
        Click(Window(-78.0, -124.8)),
        Wait(20),
        Shot("s09b-fire-off"),
    ]);

    steps.extend([
        Point(tab(2.0)),
        Wait(20),
        Shot("s10-hover-tab"),
        Click(Window(-130.85, 110.6)),
        Wait(40),
        Shot("s11-back-to-other"),
        Key(KeyCode::Escape),
        Wait(40),
        Shot("s12-closed"),
        Wait(10),
    ]);
    steps
}

fn script() -> Vec<Step> {
    if std::env::var("ECONBOX_TOUR_SCRIPT").as_deref() == Ok("settings") {
        return settings_script();
    }

    use Spot::*;
    use Step::*;

    vec![
        Wait(90),
        Shot("01-main"),
        Point(Button(TabId::Main, "clock")),
        Wait(20),
        Shot("02-hover-clock"),
        Click(Strip(TabId::Units)),
        Wait(30),
        Shot("03-units-tab"),
        Click(Button(TabId::Units, "wolf")),
        Wait(40),
        Shot("04-wolf-selected"),
        Click(Middle(300.0)),
        Wait(10),
        Click(Middle(330.0)),
        Wait(40),
        Shot("05-wolves-dropped"),
        Click(Strip(TabId::Creation)),
        Wait(20),
        Click(Button(TabId::Creation, "tile_mountains")),
        Wait(40),
        Shot("06-mountains-selected"),
        Point(Middle(260.0)),
        Wait(20),
        Shot("07-footprint"),
        Click(Canvas(25.0, 185.0)),
        Wait(40),
        Shot("08-brush-window"),
        Key(KeyCode::Escape),
        Wait(30),
        Click(Button(TabId::Creation, "clock")),
        Wait(40),
        Click(Canvas(25.0, 185.0)),
        Wait(20),
        Shot("09-speed-row"),
        Click(Canvas(25.0 + 197.8, 185.0)),
        Wait(30),
        Shot("10-speed-x5"),
        Key(KeyCode::KeyH),
        Wait(40),
        Shot("11-hidden"),
        Key(KeyCode::KeyH),
        Wait(40),
        Click(Strip(TabId::Units)),
        Wait(30),
        Drag(Canvas(900.0, 40.0), Canvas(300.0, 40.0)),
        Wait(60),
        Shot("12-units-scrolled"),
        Key(KeyCode::Tab),
        Wait(30),
        Shot("13-tab-key"),
        Click(Strip(TabId::Destruction)),
        Wait(20),
        Click(Button(TabId::Destruction, "force")),
        Wait(30),
        Shot("14-planned-button"),
        Point(Button(TabId::Destruction, "grenade")),
        Wait(3),
        Click(Button(TabId::Destruction, "grenade")),
        Wait(10),
        Point(Canvas(300.0, 300.0)),
        Wait(3),
        Click(Canvas(300.0, 300.0)),
        Wait(30),
        Shot("14b-grenade"),
        Click(Strip(TabId::Creation)),
        Wait(30),
        Down(KeyCode::ControlLeft),
        Key(KeyCode::ArrowRight),
        Wait(2),
        Key(KeyCode::ArrowRight),
        Wait(2),
        Key(KeyCode::ArrowRight),
        Up(KeyCode::ControlLeft),
        Wait(30),
        Shot("15-ctrl-arrows"),
        Down(KeyCode::ControlLeft),
        Key(KeyCode::Digit3),
        Up(KeyCode::ControlLeft),
        Wait(20),
        Shot("16-saved-slot"),
        Key(KeyCode::Escape),
        Wait(10),
        Click(Strip(TabId::Units)),
        Wait(30),
        Key(KeyCode::Digit3),
        Wait(40),
        Shot("17-slot-recall"),
        Point(Middle(260.0)),
        Down(KeyCode::AltLeft),
        Wheel(1.0),
        Wait(2),
        Wheel(1.0),
        Wait(2),
        Wheel(1.0),
        Up(KeyCode::AltLeft),
        Wait(30),
        Shot("18-alt-wheel"),
        Click(Strip(TabId::Nature)),
        Wait(30),
        Point(Canvas(400.0, 40.0)),
        Wheel(-1.0),
        Wait(2),
        Wheel(-1.0),
        Wait(2),
        Wheel(-1.0),
        Wait(2),
        Wheel(-1.0),
        Wait(40),
        Shot("19-wheel-scroll"),
        Click(Strip(TabId::Units)),
        Wait(30),
        Click(Button(TabId::Units, "pause")),
        Wait(20),
        Shot("20-paused"),
        Click(Spot::Right),
        Wait(40),
        Shot("21-arrow-page"),
        Point(Strip(TabId::Creation)),
        Wait(3),
        Press(Strip(TabId::Creation)),
        Wait(25),
        Point(Strip(TabId::Nature)),
        Wait(10),
        Point(Strip(TabId::Nature)),
        Wait(4),
        Shot("22-strip-dragging"),
        Release,
        Wait(40),
        Shot("23-strip-reordered"),
        Key(KeyCode::Tab),
        Wait(30),
        Shot("24-tab-follows-order"),
        Down(KeyCode::ControlLeft),
        Key(KeyCode::Equal),
        Wait(2),
        Key(KeyCode::Equal),
        Wait(2),
        Key(KeyCode::Equal),
        Up(KeyCode::ControlLeft),
        Wait(30),
        Shot("25-bar-size-130"),
        Key(KeyCode::KeyH),
        Wait(40),
        Down(KeyCode::ControlLeft),
        Key(KeyCode::Minus),
        Wait(2),
        Key(KeyCode::Minus),
        Wait(2),
        Key(KeyCode::Minus),
        Wait(2),
        Key(KeyCode::Minus),
        Up(KeyCode::ControlLeft),
        Wait(25),
        Shot("26-bar-size-preview"),
        Wait(90),
        Shot("27-preview-over"),
        Key(KeyCode::KeyH),
        Wait(40),
        Shot("28-bar-size-90"),
        Wait(10),
    ]
}

fn drive(
    time: Res<Time>,
    mut tour: ResMut<Tour>,
    mut commands: Commands,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut window_events: MessageWriter<WindowEvent>,
    mut buttons: MessageWriter<MouseButtonInput>,
    mut keys: MessageWriter<KeyboardInput>,
    mut wheels: MessageWriter<MouseWheel>,
    mut exit: MessageWriter<AppExit>,
    bar: Res<BarSize>,
    order: Res<StripOrder>,
) {
    let Ok((entity, mut window)) = windows.single_mut() else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());

    let point = |window: &mut Window, events: &mut MessageWriter<WindowEvent>, at: Vec2| {
        let last = window.cursor_position().unwrap_or(at);
        window.set_cursor_position(Some(at));
        events.write(WindowEvent::CursorMoved(CursorMoved {
            window: entity,
            position: at,
            delta: Some(at - last),
        }));
    };

    let press = |events: &mut MessageWriter<WindowEvent>,
                 buttons: &mut MessageWriter<MouseButtonInput>,
                 state: ButtonState| {
        let input = MouseButtonInput {
            button: MouseButton::Left,
            state,
            window: entity,
        };
        buttons.write(input);
        events.write(WindowEvent::MouseButtonInput(input));
    };

    if let Some((from, to, frame)) = tour.drag {
        let total = 20;
        let t = frame as f32 / total as f32;
        point(&mut window, &mut window_events, from.lerp(to, t));

        if frame == 0 {
            press(&mut window_events, &mut buttons, ButtonState::Pressed);
        }

        if frame >= total {
            press(&mut window_events, &mut buttons, ButtonState::Released);
            tour.drag = None;
        } else {
            tour.drag = Some((from, to, frame + 1));
        }
        return;
    }

    if tour.waiting > 0.0 {
        tour.waiting -= time.delta_secs();
        return;
    }

    let Some(step) = tour.steps.get(tour.at).cloned() else {
        exit.write(AppExit::Success);
        return;
    };
    tour.at += 1;

    match step {
        Step::Wait(frames) => tour.waiting = frames as f32 / 60.0,
        Step::Point(spot) => point(
            &mut window,
            &mut window_events,
            resolve(spot, size, *bar, &order),
        ),
        Step::Click(spot) => {
            point(
                &mut window,
                &mut window_events,
                resolve(spot, size, *bar, &order),
            );
            press(&mut window_events, &mut buttons, ButtonState::Pressed);
            let at = tour.at;
            tour.steps.insert(at, Step::Wait(2));
            tour.steps.insert(at + 1, Step::Key(KeyCode::F24));
        }
        Step::Press(spot) => {
            point(
                &mut window,
                &mut window_events,
                resolve(spot, size, *bar, &order),
            );
            press(&mut window_events, &mut buttons, ButtonState::Pressed);
        }
        Step::Release => {
            press(&mut window_events, &mut buttons, ButtonState::Released);
        }
        Step::Drag(from, to) => {
            tour.drag = Some((
                resolve(from, size, *bar, &order),
                resolve(to, size, *bar, &order),
                0,
            ));
        }
        Step::Key(KeyCode::F24) => {
            press(&mut window_events, &mut buttons, ButtonState::Released);
        }
        Step::Wheel(y) => {
            let wheel = MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y,
                window: entity,
                phase: bevy::input::touch::TouchPhase::Moved,
            };
            wheels.write(wheel);
            window_events.write(WindowEvent::MouseWheel(wheel));
        }
        Step::Down(code) | Step::Up(code) => {
            let state = if matches!(step, Step::Down(_)) {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            };
            keys.write(KeyboardInput {
                key_code: code,
                logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                state,
                text: None,
                repeat: false,
                window: entity,
            });
        }
        Step::Key(code) => {
            for state in [ButtonState::Pressed, ButtonState::Released] {
                keys.write(KeyboardInput {
                    key_code: code,
                    logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                    state,
                    text: None,
                    repeat: false,
                    window: entity,
                });
            }
        }
        Step::Shot(name) => {
            let path = tour.out.join(format!("{name}.png"));
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
            tour.waiting = 0.1;
        }
    }
}

fn main() {
    let out = std::env::var("ECONBOX_TOUR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("toolbar-tour"));
    std::fs::create_dir_all(&out).expect("make tour folder");

    let mut app = desktop::app();
    app.insert_resource(Tour {
        steps: script(),
        at: 0,
        waiting: 0.0,
        out,
        drag: None,
    })
    .add_systems(Update, drive);
    app.run();
}
