//! A drop that lands on screen plays its landing sound, softer toward the edges.

use crate::prelude::*;
use econbox_camera::{TILE_SIZE, WorldCamera};
use econbox_drops::{DropKind, Landed};
use econbox_toolbar::{Splash, Voice};

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Landed>()
            .add_message::<Splash>()
            .add_systems(Update, hear);
    }
}

pub fn voice(kind: DropKind) -> Voice {
    match kind {
        DropKind::Rain => Voice::Rain,
        DropKind::Acid => Voice::Acid,
        DropKind::Fire => Voice::Fire,
        DropKind::Lava => Voice::Lava,
        DropKind::Snow => Voice::Snow,
        DropKind::Ash => Voice::Ash,
        DropKind::MagicRain => Voice::Magic,
        DropKind::Rage => Voice::Rage,
        DropKind::LifeSeed => Voice::Seed,
    }
}

pub fn loudness(spot: Vec2, centre: Vec2, view: Vec2) -> Option<f32> {
    let half = view * 0.5 + Vec2::splat(8.0 * TILE_SIZE);
    let offset = spot - centre;
    if offset.x.abs() > half.x || offset.y.abs() > half.y {
        return None;
    }
    let depth = view.y * 0.5;
    Some(depth / (offset.length_squared() + depth * depth).sqrt())
}

fn hear(
    mut landed: MessageReader<Landed>,
    map: Option<Res<TileMap>>,
    camera: Option<Single<(&Transform, &Projection), With<WorldCamera>>>,
    mut splashes: MessageWriter<Splash>,
) {
    let (Some(map), Some(camera)) = (map, camera) else {
        landed.clear();
        return;
    };
    let (transform, projection) = camera.into_inner();
    let Projection::Orthographic(projection) = projection else {
        landed.clear();
        return;
    };
    let centre = transform.translation.truncate();
    let view = projection.area.size();
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;

    for drop in landed.read() {
        let spot = Vec2::new(
            -half.x + (drop.pos.x as f32 + 0.5) * TILE_SIZE,
            half.y - (drop.pos.y as f32 + 0.5) * TILE_SIZE,
        );
        if let Some(volume) = loudness(spot, centre, view) {
            splashes.write(Splash {
                voice: voice(drop.kind),
                volume,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_landing_is_loudest_in_the_middle_and_silent_off_screen() {
        let view = Vec2::new(400.0, 200.0);
        assert_eq!(loudness(Vec2::ZERO, Vec2::ZERO, view), Some(1.0));
        let edge = loudness(Vec2::new(200.0, 0.0), Vec2::ZERO, view).unwrap();
        assert!(edge > 0.4 && edge < 0.5, "{edge}");
        assert_eq!(loudness(Vec2::new(400.0, 0.0), Vec2::ZERO, view), None);
    }

    #[test]
    fn every_drop_has_its_own_voice() {
        let voices: Vec<Voice> = econbox_drops::DROPS.into_iter().map(voice).collect();
        for (index, voice) in voices.iter().enumerate() {
            assert!(!voices[index + 1..].contains(voice), "{voice:?}");
        }
    }
}
