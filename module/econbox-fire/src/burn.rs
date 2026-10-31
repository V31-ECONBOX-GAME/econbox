use econbox_tile::{Ground, Overlay, Tile, Wall};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burn {
    pub can_be_set_on_fire: bool,
    pub burnable: bool,
    pub burn_rate: i32,
    pub fire_chance: f32,
    pub liquid: bool,
    pub ocean: bool,
}

const BASE: Burn = Burn {
    can_be_set_on_fire: false,
    burnable: false,
    burn_rate: 10,
    fire_chance: 1.0,
    liquid: false,
    ocean: false,
};

const fn top(burnable: bool, burn_rate: i32, fire_chance: f32) -> Burn {
    Burn {
        can_be_set_on_fire: true,
        burnable,
        burn_rate,
        fire_chance,
        ..BASE
    }
}

const fn pair(burnable: bool, burn_rate: i32, low: f32, high: f32) -> [Burn; 2] {
    [
        top(burnable, burn_rate, low),
        top(burnable, burn_rate, high),
    ]
}

const SNOW: Burn = top(false, 5, 0.05);

const ICE: Burn = Burn {
    liquid: true,
    ocean: true,
    ..SNOW
};

pub fn ground(ground: Ground) -> Burn {
    match ground {
        Ground::DeepOcean | Ground::CloseOcean | Ground::ShallowWaters => Burn {
            liquid: true,
            ocean: true,
            ..BASE
        },
        Ground::Lava2 | Ground::Lava3 => Burn {
            liquid: true,
            ..BASE
        },
        Ground::Sand
        | Ground::SoilLow
        | Ground::SoilHigh
        | Ground::Hills
        | Ground::Mountains
        | Ground::Summit => Burn {
            can_be_set_on_fire: true,
            ..BASE
        },
    }
}

pub fn overlay(overlay: Overlay) -> Option<[Burn; 2]> {
    Some(match overlay {
        Overlay::Bare => return None,
        Overlay::Grass => pair(true, 5, 0.05, 0.06),
        Overlay::Jungle => pair(true, 5, 0.04, 0.05),
        Overlay::Enchanted => pair(true, 4, 0.03, 0.04),
        Overlay::Infernal => pair(false, 5, 0.05, 0.06),
        Overlay::Pumpkin => {
            [Burn {
                burnable: true,
                fire_chance: 0.06,
                ..BASE
            }; 2]
        }
        Overlay::Savanna => pair(true, 5, 0.06, 0.06),
        Overlay::Birch | Overlay::Maple | Overlay::Mushroom | Overlay::Clover => {
            pair(true, 4, 0.03, 0.03)
        }
        Overlay::Swamp | Overlay::Desert | Overlay::Crystal | Overlay::Frost => {
            pair(false, 5, 0.05, 0.06)
        }
        Overlay::Blight => pair(true, 1, 0.02, 0.02),
        Overlay::Candy => pair(true, 4, 0.05, 0.06),
        Overlay::Garlic => pair(true, 5, 0.03, 0.03),
        Overlay::Lemon => pair(true, 2, 0.05, 0.06),
        Overlay::Rockland => pair(true, 3, 0.03, 0.03),
        Overlay::Flower => [top(true, 5, 0.03), top(true, 4, 0.03)],
        Overlay::Starlight | Overlay::Paradox => pair(true, 2, 0.03, 0.03),
        Overlay::Void => pair(false, 1, 0.03, 0.03),
    })
}

pub fn wall(wall: Wall) -> Burn {
    match wall {
        Wall::Wild => Burn {
            burnable: true,
            burn_rate: 5,
            ..BASE
        },
        Wall::Green => Burn {
            can_be_set_on_fire: true,
            burnable: true,
            burn_rate: 5,
            ..BASE
        },
        Wall::Order | Wall::Evil | Wall::Ancient | Wall::Iron | Wall::Light => BASE,
    }
}

pub fn of(tile: Tile) -> Burn {
    if tile.frozen {
        return if tile.ground.is_water() { ICE } else { SNOW };
    }
    if let Some(kind) = tile.wall {
        return wall(kind);
    }
    match overlay(tile.overlay) {
        Some(pair) => pair[usize::from(tile.ground.is_high())],
        None => ground(tile.ground),
    }
}

pub fn has_top(tile: Tile) -> bool {
    tile.wall.is_some() || tile.overlay != Overlay::Bare
}

pub fn object_burns(tile: Tile) -> bool {
    tile.flora.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grass_burns_at_five_and_catches_one_time_in_twenty() {
        let low = of(Tile::new(Ground::SoilLow, Overlay::Grass));
        let high = of(Tile::new(Ground::SoilHigh, Overlay::Grass));

        assert!(low.burnable && low.can_be_set_on_fire);
        assert_eq!(low.burn_rate, 5);
        assert!((low.fire_chance - 0.05).abs() < 1e-6);
        assert!((high.fire_chance - 0.06).abs() < 1e-6);
    }

    #[test]
    fn bare_ground_scorches_but_never_burns_and_water_is_liquid() {
        let sand = of(Tile::bare(Ground::Sand));
        assert!(sand.can_be_set_on_fire && !sand.burnable);
        assert_eq!(sand.fire_chance, 1.0);

        let sea = of(Tile::bare(Ground::ShallowWaters));
        assert!(sea.liquid && sea.ocean);
        let lava = of(Tile::bare(Ground::Lava3));
        assert!(lava.liquid && !lava.ocean && !lava.can_be_set_on_fire);
    }

    #[test]
    fn deserts_swamps_and_frost_do_not_burn() {
        for kind in [
            Overlay::Desert,
            Overlay::Swamp,
            Overlay::Frost,
            Overlay::Void,
        ] {
            assert!(!of(Tile::new(Ground::Sand, kind)).burnable, "{kind:?}");
        }
    }

    #[test]
    fn only_wild_and_green_walls_burn() {
        let walled = |kind| Tile {
            wall: Some(kind),
            ..Tile::bare(Ground::SoilLow)
        };
        assert!(of(walled(Wall::Wild)).burnable);
        assert!(of(walled(Wall::Green)).burnable);
        assert!(!of(walled(Wall::Iron)).burnable);
        assert!(!of(walled(Wall::Wild)).can_be_set_on_fire);
    }

    #[test]
    fn frozen_land_is_snow() {
        let frozen = Tile {
            frozen: true,
            ..Tile::new(Ground::SoilLow, Overlay::Grass)
        };
        assert_eq!(of(frozen), SNOW);
    }
}
