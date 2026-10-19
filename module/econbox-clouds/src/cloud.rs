use crate::art::Puff;
use crate::dice::Dice;
use crate::kind::CloudKind;
use bevy::prelude::*;
use econbox_tilemap::TilePos;

pub const FADE: f32 = 0.2;
pub const FIRST_DROP: f32 = 3.0;
pub const FULL_ZOOM: f32 = 100.0;
pub const DROP_REACH: Vec2 = Vec2::new(0.4, 0.2);
pub const SHADOW_OFFSET: Vec2 = Vec2::new(0.0, -20.0);
pub const SPAWN_LEFT: i32 = -50;
pub const SPAWN_RIGHT: i32 = 30;
pub const SPAWN_ABOVE: i32 = 25;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Cloud {
    pub kind: CloudKind,
    pub puff: Puff,
    pub at: Vec2,
    pub speed: f32,
    pub flip: bool,
    pub alpha: f32,
    pub alive: f32,
    pub lifespan: f32,
    pub dying: bool,
    pub drop_in: f32,
    pub bolt_in: f32,
    pub serial: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Deeds {
    pub drops: u32,
    pub bolts: u32,
    pub gone: bool,
}

impl Cloud {
    pub fn new(kind: CloudKind, puff: Puff, at: Vec2, speed: f32, flip: bool, serial: u64) -> Self {
        Self {
            kind,
            puff,
            at,
            speed,
            flip,
            alpha: 0.0,
            alive: 0.0,
            lifespan: 0.0,
            dying: false,
            drop_in: kind.drop_every(),
            bolt_in: 0.0,
            serial,
        }
    }

    pub fn ceiling(&self, ortho: Option<f32>) -> f32 {
        let top = self.kind.max_alpha();

        match ortho {
            Some(size) if size > 0.0 => (top * size / FULL_ZOOM).min(top),
            _ => top,
        }
    }

    pub fn reach(&self) -> Vec2 {
        let (width, height) = self.puff.tiles();
        Vec2::new(width, height) * DROP_REACH
    }

    pub fn update(&mut self, dt: f32, running: bool, width: f32, ortho: Option<f32>) -> Deeds {
        let mut deeds = Deeds::default();
        self.alive += dt;

        if running {
            self.at.x += self.speed * dt;

            if self.drop_in > 0.0 {
                self.drop_in -= dt;
            } else {
                self.drop_in = self.kind.drop_every();
                if self.alive >= FIRST_DROP {
                    deeds.drops += 1;
                }
            }

            if let Some(every) = self.kind.bolt_every() {
                if self.bolt_in > 0.0 {
                    self.bolt_in -= dt;
                } else {
                    self.bolt_in = every;
                    deeds.bolts += 1;
                }
            }
        }

        if self.at.x > width || (self.lifespan > 0.0 && self.alive > self.lifespan) {
            self.dying = true;
        }

        if self.dying {
            if self.alpha > 0.0 {
                self.alpha -= dt * FADE;
            } else {
                self.alpha = 0.0;
                deeds.gone = true;
            }
        } else {
            let ceiling = self.ceiling(ortho);

            if self.alpha < ceiling {
                self.alpha = (self.alpha + dt * FADE).min(ceiling);
            } else if self.alpha > ceiling {
                self.alpha = (self.alpha - dt * FADE).max(ceiling);
            }
        }

        deeds
    }

    pub fn drop_spot(&self, dice: &mut Dice) -> IVec2 {
        let reach = self.reach();
        let x = dice.range(-reach.x, reach.x) as i32;

        IVec2::new(
            self.at.x as i32 + x,
            self.at.y as i32 + self.shadow_row(dice),
        )
    }

    pub fn bolt_spot(&self, dice: &mut Dice) -> IVec2 {
        let reach = self.reach();
        let x = dice.range(reach.x * 0.5, reach.x) as i32;

        IVec2::new(
            self.at.x as i32 + x,
            self.at.y as i32 + self.shadow_row(dice),
        )
    }

    fn shadow_row(&self, dice: &mut Dice) -> i32 {
        let reach = self.reach();
        dice.range(-reach.y + SHADOW_OFFSET.y, reach.y + SHADOW_OFFSET.y) as i32
    }
}

pub fn spawn_spot(dice: &mut Dice, height: f32) -> Vec2 {
    Vec2::new(
        dice.int(SPAWN_LEFT, SPAWN_RIGHT) as f32,
        dice.int(0, height as i32 + SPAWN_ABOVE) as f32,
    )
}

pub fn over_tile(pos: TilePos, height: u32) -> Vec2 {
    let row = height.saturating_sub(1 + pos.y);
    Vec2::new(pos.x as f32, row as f32 - SHADOW_OFFSET.y)
}

pub fn spot_tile(spot: IVec2, width: u32, height: u32) -> Option<TilePos> {
    let inside = spot.x >= 0 && spot.y >= 0 && (spot.x as u32) < width && (spot.y as u32) < height;
    inside.then(|| TilePos::new(spot.x as u32, height - 1 - spot.y as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 0.02;

    fn plain(at: Vec2, speed: f32) -> Cloud {
        Cloud::new(CloudKind::Normal, Puff::Broad, at, speed, false, 1)
    }

    fn run(
        cloud: &mut Cloud,
        seconds: f32,
        running: bool,
        width: f32,
        ortho: Option<f32>,
    ) -> Deeds {
        let mut total = Deeds::default();

        for _ in 0..(seconds / FRAME).round() as u32 {
            let deeds = cloud.update(FRAME, running, width, ortho);
            total.drops += deeds.drops;
            total.bolts += deeds.bolts;
            total.gone |= deeds.gone;

            if deeds.gone {
                break;
            }
        }

        total
    }

    #[test]
    fn a_cloud_drifts_east_at_its_own_speed() {
        let mut cloud = plain(Vec2::new(0.0, 40.0), 3.0);
        run(&mut cloud, 10.0, true, 1000.0, None);

        assert!((cloud.at.x - 30.0).abs() < 0.01, "{}", cloud.at.x);
        assert_eq!(cloud.at.y, 40.0);
    }

    #[test]
    fn a_paused_world_holds_the_cloud_but_lets_it_fade_in() {
        let mut cloud = plain(Vec2::new(5.0, 40.0), 3.0);
        run(&mut cloud, 1.0, false, 1000.0, None);

        assert_eq!(cloud.at.x, 5.0);
        assert!((cloud.alpha - FADE).abs() < 1e-3, "{}", cloud.alpha);
    }

    #[test]
    fn it_fades_in_at_a_fifth_per_second_up_to_its_ceiling() {
        let mut cloud = plain(Vec2::ZERO, 2.0);
        run(&mut cloud, 1.0, true, 1000.0, None);
        assert!((cloud.alpha - 0.2).abs() < 1e-3);

        run(&mut cloud, 10.0, true, 1000.0, None);
        assert_eq!(cloud.alpha, CloudKind::Normal.max_alpha());
    }

    #[test]
    fn zooming_in_closer_than_a_hundred_tiles_thins_the_cloud_out() {
        let cloud = plain(Vec2::ZERO, 2.0);

        assert_eq!(cloud.ceiling(Some(250.0)), 0.5);
        assert_eq!(cloud.ceiling(Some(100.0)), 0.5);
        assert!((cloud.ceiling(Some(50.0)) - 0.25).abs() < 1e-6);
        assert_eq!(cloud.ceiling(None), 0.5);

        let mut seen = cloud.clone();
        run(&mut seen, 5.0, true, 1000.0, Some(250.0));
        run(&mut seen, 1.0, true, 1000.0, Some(20.0));
        assert!((seen.alpha - 0.3).abs() < 1e-3, "{}", seen.alpha);
        run(&mut seen, 5.0, true, 1000.0, Some(20.0));
        assert!((seen.alpha - 0.1).abs() < 1e-3, "{}", seen.alpha);
    }

    #[test]
    fn past_the_right_edge_it_fades_out_and_is_gone() {
        let mut cloud = plain(Vec2::new(220.0, 30.0), 6.0);
        run(&mut cloud, 5.0, true, 256.0, None);
        assert!(!cloud.dying);

        let deeds = run(&mut cloud, 1.5, true, 256.0, None);
        assert!(cloud.dying);
        assert!(!deeds.gone);

        let deeds = run(&mut cloud, 5.0, true, 256.0, None);
        assert!(deeds.gone);
        assert_eq!(cloud.alpha, 0.0);
    }

    #[test]
    fn a_lifespan_ends_the_cloud_early() {
        let mut cloud = plain(Vec2::ZERO, 2.0);
        cloud.lifespan = 4.0;
        run(&mut cloud, 3.9, true, 1000.0, None);
        assert!(!cloud.dying);

        run(&mut cloud, 0.2, true, 1000.0, None);
        assert!(cloud.dying);
    }

    #[test]
    fn nothing_drops_in_the_first_three_seconds() {
        let mut cloud = Cloud::new(CloudKind::Rain, Puff::Broad, Vec2::ZERO, 2.0, false, 1);
        assert_eq!(run(&mut cloud, 2.9, true, 1000.0, None).drops, 0);
        assert!(run(&mut cloud, 1.0, true, 1000.0, None).drops > 0);
    }

    #[test]
    fn a_rain_cloud_drops_every_few_frames_and_a_plain_one_every_two_seconds() {
        let mut rain = Cloud::new(CloudKind::Rain, Puff::Broad, Vec2::ZERO, 2.0, false, 1);
        let mut plain = plain(Vec2::ZERO, 2.0);
        run(&mut rain, 3.0, true, 1000.0, None);
        run(&mut plain, 3.0, true, 1000.0, None);

        let rained = run(&mut rain, 10.0, true, 1000.0, None).drops;
        let seeded = run(&mut plain, 10.0, true, 1000.0, None).drops;

        assert!((120..=170).contains(&rained), "{rained}");
        assert!((4..=5).contains(&seeded), "{seeded}");
    }

    #[test]
    fn a_held_world_drops_nothing() {
        let mut cloud = Cloud::new(CloudKind::Lightning, Puff::Broad, Vec2::ZERO, 2.0, false, 1);
        let deeds = run(&mut cloud, 10.0, false, 1000.0, None);

        assert_eq!((deeds.drops, deeds.bolts), (0, 0));
    }

    #[test]
    fn drops_land_under_the_shadow_inside_most_of_the_width() {
        let cloud = Cloud::new(
            CloudKind::Rain,
            Puff::Broad,
            Vec2::new(100.0, 100.0),
            2.0,
            false,
            1,
        );
        let mut dice = Dice::new(3);
        let reach = cloud.reach();

        assert_eq!(reach, Vec2::new(93.0 * 0.4, 30.0 * 0.2));

        for _ in 0..2000 {
            let spot = cloud.drop_spot(&mut dice);
            assert!((spot.x - 100).abs() <= reach.x as i32 + 1);
            assert!(
                spot.y >= 100 - 20 - reach.y as i32 - 1 && spot.y <= 100 - 20 + reach.y as i32 + 1
            );

            let bolt = cloud.bolt_spot(&mut dice);
            assert!(bolt.x >= 100 + (reach.x * 0.5) as i32 - 1 && bolt.x <= 100 + reach.x as i32);
        }
    }

    #[test]
    fn fresh_clouds_enter_from_the_west_anywhere_up_the_map() {
        let mut dice = Dice::new(11);
        let mut top: f32 = 0.0;

        for _ in 0..4000 {
            let spot = spawn_spot(&mut dice, 256.0);
            assert!(spot.x >= -50.0 && spot.x < 30.0);
            assert!(spot.y >= 0.0 && spot.y < 256.0 + 25.0);
            top = top.max(spot.y);
        }

        assert!(top > 256.0);
    }

    #[test]
    fn a_cloud_called_onto_a_tile_casts_its_shadow_there() {
        let at = over_tile(TilePos::new(40, 200), 256);

        assert_eq!(at, Vec2::new(40.0, 55.0 + 20.0));
        assert_eq!(
            spot_tile(IVec2::new(40, 55), 256, 256),
            Some(TilePos::new(40, 200))
        );
    }

    #[test]
    fn spots_off_the_map_have_no_tile() {
        assert_eq!(spot_tile(IVec2::new(-1, 3), 64, 64), None);
        assert_eq!(spot_tile(IVec2::new(3, 64), 64, 64), None);
        assert_eq!(
            spot_tile(IVec2::new(0, 0), 64, 64),
            Some(TilePos::new(0, 63))
        );
    }
}
