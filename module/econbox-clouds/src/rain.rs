use crate::dice::Dice;
use econbox_brush::{Power, Shape, Touch, cells};
use econbox_creature::{Effect, Strike};
use econbox_tile::Tile;
use econbox_tilemap::{TileMap, TilePos};

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
    use econbox_tile::{Ground, Overlay};

    #[test]
    fn a_bolt_scorches_a_small_circle_and_kills_inside_it() {
        let map = TileMap::filled(16, 16, Tile::new(Ground::SoilLow, Overlay::Grass));
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
}
