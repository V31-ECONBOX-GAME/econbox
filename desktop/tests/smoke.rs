#[test]
fn headless_app_runs() {
    let mut app = econbox::headless_app();

    for _ in 0..100 {
        app.update();
    }
}
