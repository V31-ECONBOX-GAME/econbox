use crate::dice::Dice;
use crate::kind::Drop;
use econbox_brush::{Power, Shape, Touch, cells};
use econbox_creature::{Effect, Strike};
use econbox_tile::{Ground, Tile};
use econbox_tilemap::{TileMap, TilePos};

pub const ACID_HURT: f32 = 20.0;
pub const ACID_SPARE: f32 = 0.6;
pub const ACID_REACH: f32 = 3.0;
pub const BOLT_ODDS: f32 = 0.01;
pub const BOLT_REACH: u32 = 2;

#[derive(Debug, Clone, Default)]
pub struct Fall {
    pub changes: Vec<(TilePos, Tile)>,
    pub strikes: Vec<Strike>,
}

impl Fall {
    fn touch(&mut self, map: &TileMap, pos: TilePos, power: Power, touch: Touch) {
        if let Some(tile) = map.get(pos)
            && let Some(next) = power.touched(tile, touch)
        {
            self.changes.push((pos, next));
        }
    }
}

pub fn inert(drop: Drop) -> bool {
    matches!(
        drop,
        Drop::Rage | Drop::Ash | Drop::MagicRain | Drop::LifeSeed
    )
}

fn beside(map: &TileMap, pos: TilePos) -> impl Iterator<Item = TilePos> + '_ {
    [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .filter_map(move |(dx, dy)| {
            let x = u32::try_from(pos.x as i32 + dx).ok()?;
            let y = u32::try_from(pos.y as i32 + dy).ok()?;
            (x < map.width && y < map.height).then_some(TilePos::new(x, y))
        })
}

pub fn land(drop: Drop, map: &TileMap, pos: TilePos, dice: &mut Dice) -> Fall {
    let mut fall = Fall::default();
    let touch = Touch {
        rim: 0.0,
        noise: dice.roll(),
    };

    match drop {
        Drop::Rain => {
            for near in beside(map, pos) {
                fall.touch(map, near, Power::Rain, touch);
            }
        }
        Drop::Acid => {
            fall.touch(map, pos, Power::Acid, touch);
            if !dice.chance(ACID_SPARE) {
                fall.strikes.push(Strike {
                    centre: pos,
                    reach: ACID_REACH,
                    effect: Effect::Hurt(ACID_HURT),
                });
            }
        }
        Drop::Fire => fall.touch(map, pos, Power::Fire, touch),
        Drop::Snow => fall.touch(map, pos, Power::Frost, touch),
        Drop::Lava => {
            if let Some(tile) = map.get(pos)
                && tile.ground.is_land()
                && !tile.ground.is_lava()
            {
                fall.changes.push((pos, Tile::bare(Ground::Lava3)));
            }
        }
        Drop::Rage | Drop::Ash | Drop::MagicRain | Drop::LifeSeed => {}
    }

    fall
}

pub fn bolt(map: &TileMap, centre: TilePos, dice: &mut Dice) -> Fall {
    let mut fall = Fall::default();
    let reach = BOLT_REACH as f32;

    for pos in cells(map, centre, Shape::Round, BOLT_REACH) {
        let dx = pos.x as f32 - centre.x as f32;
        let dy = pos.y as f32 - centre.y as f32;
        let touch = Touch {
            rim: (dx.hypot(dy) / reach).min(1.0),
            noise: dice.roll(),
        };
        fall.touch(map, pos, Power::Lightning, touch);
    }

    fall.strikes.push(Strike {
        centre,
        reach,
        effect: Effect::Kill,
    });

    fall
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::Overlay;

    fn field(ground: Ground, overlay: Overlay) -> TileMap {
        TileMap::filled(16, 16, Tile::new(ground, overlay))
    }

    #[test]
    fn rain_falls_on_the_tile_and_its_four_neighbours() {
        let map = field(Ground::Sand, Overlay::Bare);
        let fall = land(Drop::Rain, &map, TilePos::new(5, 5), &mut Dice::new(1));
        let mut spots: Vec<TilePos> = fall.changes.iter().map(|(pos, _)| *pos).collect();
        spots.sort_by_key(|pos| (pos.y, pos.x));

        assert_eq!(
            spots,
            [
                TilePos::new(5, 4),
                TilePos::new(4, 5),
                TilePos::new(5, 5),
                TilePos::new(6, 5),
                TilePos::new(5, 6),
            ]
        );
        assert!(
            fall.changes
                .iter()
                .all(|(_, tile)| tile.ground == Ground::SoilLow)
        );
    }

    #[test]
    fn rain_at_the_map_corner_stays_on_the_map() {
        let map = field(Ground::Sand, Overlay::Bare);
        let fall = land(Drop::Rain, &map, TilePos::new(0, 0), &mut Dice::new(1));

        assert_eq!(fall.changes.len(), 3);
    }

    #[test]
    fn acid_strips_the_ground_and_sometimes_burns_who_stands_near() {
        let map = field(Ground::SoilLow, Overlay::Grass);
        let mut dice = Dice::new(5);
        let mut burns = 0;

        for _ in 0..1000 {
            let fall = land(Drop::Acid, &map, TilePos::new(3, 3), &mut dice);
            assert_eq!(
                fall.changes,
                [(TilePos::new(3, 3), Tile::bare(Ground::SoilLow))]
            );
            burns += fall.strikes.len();
        }

        assert!((330..470).contains(&burns), "{burns}");
    }

    #[test]
    fn lava_drops_melt_land_but_not_the_sea() {
        let land_map = field(Ground::Hills, Overlay::Bare);
        let sea = field(Ground::DeepOcean, Overlay::Bare);
        let mut dice = Dice::new(2);

        assert_eq!(
            land(Drop::Lava, &land_map, TilePos::new(1, 1), &mut dice).changes,
            [(TilePos::new(1, 1), Tile::bare(Ground::Lava3))]
        );
        assert!(
            land(Drop::Lava, &sea, TilePos::new(1, 1), &mut dice)
                .changes
                .is_empty()
        );
    }

    #[test]
    fn a_bolt_scorches_a_small_circle_and_kills_inside_it() {
        let map = field(Ground::SoilLow, Overlay::Grass);
        let fall = bolt(&map, TilePos::new(8, 8), &mut Dice::new(4));

        assert!(!fall.changes.is_empty());
        assert!(
            fall.changes
                .iter()
                .all(|(_, tile)| tile.overlay == Overlay::Infernal)
        );
        assert_eq!(fall.strikes.len(), 1);
        assert_eq!(fall.strikes[0].effect, Effect::Kill);
    }

    #[test]
    fn rage_ash_magic_and_life_seeds_change_nothing_yet() {
        let map = field(Ground::SoilLow, Overlay::Grass);

        for drop in [Drop::Rage, Drop::Ash, Drop::MagicRain, Drop::LifeSeed] {
            let fall = land(drop, &map, TilePos::new(2, 2), &mut Dice::new(9));
            assert!(fall.changes.is_empty() && fall.strikes.is_empty());
            assert!(inert(drop));
        }
    }
}
