use bevy::prelude::*;
use entropybox_finance::FinancePlugin;

#[test]
fn plugin_builds() {
    let mut app = App::new();
    app.add_plugins(FinancePlugin);
    app.update();
}
