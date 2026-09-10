use bevy::prelude::*;

pub fn run(ticks: usize) -> App {
    let mut app = entropybox::headless_app();

    for _ in 0..ticks {
        app.update();
    }
    app
}
