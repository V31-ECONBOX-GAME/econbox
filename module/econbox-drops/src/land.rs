use crate::kind::DropKind;
use econbox_brush::{Power, Touch};
use econbox_creature::{Effect, Strike};
use econbox_noise::Randy;
use econbox_tile::{Ground, Tile};
use econbox_tilemap::{TileMap, TilePos};

pub const ACID_HURT: f32 = 20.0;
pub const ACID_SPARE: f32 = 0.6;
pub const ACID_REACH: f32 = 3.0;

#[derive(Debug, Clone, Default)]
pub struct Landing {
    pub changes: Vec<(TilePos, Tile)>,
    pub strikes: Vec<Strike>,
}

impl Landing {
    fn touch(&mut self, map: &TileMap, pos: TilePos, power: Power, touch: Touch) {
        if let Some(tile) = map.get(pos)
            && let Some(next) = power.touched(tile, touch)
        {
            self.changes.push((pos, next));
        }
    }
}

pub fn beside(map: &TileMap, pos: TilePos) -> impl Iterator<Item = TilePos> + '_ {
    [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .filter_map(move |(dx, dy)| {
            let x = u32::try_from(pos.x as i32 + dx).ok()?;
            let y = u32::try_from(pos.y as i32 + dy).ok()?;
            (x < map.width && y < map.height).then_some(TilePos::new(x, y))
        })
}

pub fn land(kind: DropKind, map: &TileMap, pos: TilePos, randy: &mut Randy) -> Landing {
    let mut landing = Landing::default();
    let touch = Touch {
        rim: 0.0,
        noise: randy.random_int(0, i32::MAX) as u32,
    };

    match kind {
        DropKind::Rain => {
            for near in beside(map, pos) {
                landing.touch(map, near, Power::Rain, touch);
            }
        }
        DropKind::Acid => {
            landing.touch(map, pos, Power::Acid, touch);
            if !randy.random_chance(ACID_SPARE) {
                landing.strikes.push(Strike {
                    centre: pos,
                    reach: ACID_REACH,
                    effect: Effect::Hurt(ACID_HURT),
                });
            }
        }
        DropKind::Fire => landing.touch(map, pos, Power::Fire, touch),
        DropKind::Snow => landing.touch(map, pos, Power::Frost, touch),
        DropKind::Lava => {
            if let Some(tile) = map.get(pos)
                && !tile.ground.is_lava()
            {
                landing.changes.push((pos, Tile::bare(Ground::Lava3)));
            }
        }
        DropKind::Ash | DropKind::MagicRain | DropKind::Rage | DropKind::LifeSeed => {}
    }

    landing
}
