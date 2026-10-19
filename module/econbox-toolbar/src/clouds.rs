use crate::catalog::Def;
use crate::{Chosen, Cue};
use bevy::prelude::*;
use econbox_brush::{PointerOverUi, SPAWN_GAP};
use econbox_clouds::SpawnCloud;
use econbox_cursor::Hovered;

fn many(keys: Option<Res<ButtonInput<KeyCode>>>) -> bool {
    keys.is_some_and(|keys| keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]))
}

pub fn summon(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut armed: Local<bool>,
    chosen: Res<Chosen>,
    hovered: Res<Hovered>,
    over_ui: Res<PointerOverUi>,
    time: Res<Time>,
    mut rest: Local<f32>,
    mut clouds: MessageWriter<SpawnCloud>,
    mut cues: MessageWriter<Cue>,
) {
    let Some(kind) = chosen.def().and_then(Def::cloud) else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        *rest = 0.0;
        *armed = !over_ui.0;
    }

    *rest = (*rest - time.delta_secs()).max(0.0);

    if over_ui.0 || !*armed || !buttons.pressed(MouseButton::Left) || *rest > 0.0 {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) && !many(keys) {
        return;
    }

    let Some(pos) = hovered.pos else { return };

    clouds.write(SpawnCloud { kind, pos });
    cues.write(Cue::Cloud);
    *rest = SPAWN_GAP;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Act, TabId, buttons};
    use crate::find;
    use bevy::time::TimeUpdateStrategy;
    use econbox_clouds::{CloudKind, KINDS};
    use econbox_tilemap::TilePos;
    use std::time::Duration;

    const CLOUDS: [(&str, CloudKind); 10] = [
        ("cloud", CloudKind::Normal),
        ("cloud_rain", CloudKind::Rain),
        ("cloud_lightning", CloudKind::Lightning),
        ("cloud_fire", CloudKind::Fire),
        ("cloud_acid", CloudKind::Acid),
        ("cloud_lava", CloudKind::Lava),
        ("cloud_ash", CloudKind::Ash),
        ("cloud_magic", CloudKind::Magic),
        ("cloud_rage", CloudKind::Rage),
        ("cloud_snow", CloudKind::Snow),
    ];

    fn app(chosen: Option<&'static str>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                50,
            )))
            .insert_resource(Chosen(chosen))
            .insert_resource(Hovered {
                pos: Some(TilePos::new(7, 9)),
                tile: None,
            })
            .init_resource::<PointerOverUi>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<SpawnCloud>()
            .add_message::<Cue>()
            .add_systems(Update, summon);
        app.update();
        app
    }

    fn frame(app: &mut App) -> Vec<SpawnCloud> {
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        let messages = app.world().resource::<Messages<SpawnCloud>>();
        let called: Vec<SpawnCloud> = messages.iter_current_update_messages().copied().collect();
        let heard = app.world().resource::<Messages<Cue>>();
        let puffs = heard
            .iter_current_update_messages()
            .filter(|cue| **cue == Cue::Cloud)
            .count();
        assert_eq!(puffs, called.len());
        called
    }

    fn press(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
    }

    #[test]
    fn every_cloud_button_calls_the_cloud_worldbox_gives_its_power() {
        for (id, kind) in CLOUDS {
            let def = find(id).expect("a cloud button");

            assert_eq!(def.act, Act::Cloud(kind), "{id}");
            assert_eq!(def.cloud(), Some(kind), "{id}");
            assert!(def.selectable(), "{id}");
        }

        for (id, kind) in CLOUDS.into_iter().skip(1) {
            assert_eq!(CloudKind::from_id(id), Some(kind), "{id}");
        }

        assert_eq!(CloudKind::from_id("cloud"), None);
        assert_eq!(CloudKind::Normal.id(), "cloud_normal");
    }

    #[test]
    fn every_kind_of_cloud_has_exactly_one_button_on_the_nature_tab() {
        let called: Vec<CloudKind> = buttons(TabId::Nature).filter_map(Def::cloud).collect();

        assert_eq!(called.len(), KINDS.len());

        for kind in KINDS {
            assert_eq!(called.iter().filter(|&&found| found == kind).count(), 1);
        }
    }

    #[test]
    fn a_click_on_the_map_calls_the_chosen_cloud_over_the_hovered_tile_once() {
        let mut app = app(Some("cloud_acid"));
        press(&mut app);

        assert_eq!(
            frame(&mut app),
            [SpawnCloud {
                kind: CloudKind::Acid,
                pos: TilePos::new(7, 9),
            }]
        );

        for _ in 0..10 {
            assert!(frame(&mut app).is_empty());
        }
    }

    #[test]
    fn holding_shift_keeps_calling_clouds_a_spawn_gap_apart() {
        let mut app = app(Some("cloud"));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        press(&mut app);

        let called: usize = (0..20).map(|_| frame(&mut app).len()).sum();

        assert!((7..=10).contains(&called), "{called}");
    }

    #[test]
    fn a_press_that_starts_on_the_interface_never_calls_a_cloud() {
        let mut app = app(Some("cloud_rain"));
        app.world_mut().resource_mut::<PointerOverUi>().0 = true;
        press(&mut app);
        assert!(frame(&mut app).is_empty());

        app.world_mut().resource_mut::<PointerOverUi>().0 = false;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);

        for _ in 0..10 {
            assert!(frame(&mut app).is_empty());
        }
    }

    #[test]
    fn no_cloud_comes_without_a_cloud_power_or_off_the_map() {
        let mut other = app(Some("tile_sand"));
        press(&mut other);
        assert!(frame(&mut other).is_empty());

        let mut off = app(Some("cloud_snow"));
        off.world_mut().resource_mut::<Hovered>().pos = None;
        press(&mut off);
        assert!(frame(&mut off).is_empty());
    }
}
