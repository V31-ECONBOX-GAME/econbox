use crate::kind::DropKind;
use econbox_creature::{Afflict, Census, Harm, Relieve, Spawn, Species, Status};
use econbox_noise::Randy;
use econbox_temperature::{Frost, Touched, freeze, thaw};
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::{Emit, Glow, TileMap, TilePos};

pub const TERRAFORM_FLASH: i32 = 20;
pub const UNIT_REACH: f32 = 3.0;
pub const ACID_HURT: f32 = 20.0;
pub const ACID_SPARE: f32 = 0.6;
pub const SNOW_TRIES: usize = 10;
pub const SNOW_SPREAD: f32 = 11.0;
pub const CHUNK_SIZE: u32 = 16;
pub const CROWD: usize = 4;
pub const SAME_STRENGTH_ODDS: f32 = 0.05;
pub const ACID_SMOKE_ODDS: f32 = 0.2;
pub const FIRE_GLOW_ODDS: f32 = 0.2;

#[derive(Debug, Clone, Default)]
pub struct Landing {
    pub cells: Vec<TilePos>,
    pub flashes: Vec<(TilePos, i32)>,
    pub ignite: Vec<TilePos>,
    pub douse: Vec<TilePos>,
    pub emit: Vec<Emit>,
    pub afflict: Vec<Afflict>,
    pub relieve: Vec<Relieve>,
    pub harm: Vec<Harm>,
    pub spawn: Vec<Spawn>,
}

impl Landing {
    fn set(&mut self, map: &mut TileMap, pos: TilePos, tile: Tile) {
        if map.get(pos) != Some(tile) {
            map.set(pos.x, pos.y, tile);
            self.cells.push(pos);
            self.flashes.push((pos, TERRAFORM_FLASH));
        }
    }

    fn take(&mut self, touched: Touched) {
        self.cells.extend(touched.cells);
        self.flashes.extend(
            touched
                .flashes
                .into_iter()
                .map(|pos| (pos, econbox_temperature::FREEZE_FLASH)),
        );
    }

    fn afflict(&mut self, pos: TilePos, status: Status, chance: f32) {
        self.afflict.push(Afflict {
            centre: pos,
            reach: UNIT_REACH,
            status,
            chance,
        });
    }
}

pub fn neighbours(map: &TileMap, pos: TilePos) -> Vec<TilePos> {
    [(0, -1), (-1, 0), (1, 0), (0, 1)]
        .into_iter()
        .filter_map(|(dx, dy)| {
            let x = u32::try_from(pos.x as i32 + dx).ok()?;
            let y = u32::try_from(pos.y as i32 + dy).ok()?;
            (x < map.width && y < map.height).then_some(TilePos::new(x, y))
        })
        .collect()
}

pub fn spreads_by_water(overlay: Overlay) -> bool {
    matches!(
        overlay,
        Overlay::Grass
            | Overlay::Savanna
            | Overlay::Jungle
            | Overlay::Desert
            | Overlay::Lemon
            | Overlay::Swamp
            | Overlay::Candy
            | Overlay::Mushroom
            | Overlay::Birch
            | Overlay::Maple
            | Overlay::Garlic
            | Overlay::Flower
            | Overlay::Clover
    )
}

pub fn grow_strength(overlay: Overlay) -> i32 {
    match overlay {
        Overlay::Grass
        | Overlay::Birch
        | Overlay::Maple
        | Overlay::Rockland
        | Overlay::Garlic
        | Overlay::Flower
        | Overlay::Starlight
        | Overlay::Clover
        | Overlay::Void
        | Overlay::Paradox => 5,
        _ => 6,
    }
}

pub fn spawn_pot(tile: Tile) -> &'static [(Species, u32)] {
    if tile.frozen || tile.wall.is_some() {
        return &[];
    }

    if tile.ground == Ground::Sand && tile.overlay == Overlay::Bare {
        return &[(Species::Crab, 1)];
    }

    if !tile.ground.is_soil() {
        return &[];
    }

    match tile.overlay {
        Overlay::Grass => &[
            (Species::Wolf, 1),
            (Species::Fox, 2),
            (Species::Raccoon, 1),
            (Species::Sheep, 3),
            (Species::Chicken, 3),
            (Species::Rabbit, 3),
            (Species::Fly, 4),
            (Species::Beetle, 4),
            (Species::Grasshopper, 4),
        ],
        Overlay::Savanna => &[
            (Species::Rhino, 1),
            (Species::Hyena, 1),
            (Species::Buffalo, 1),
            (Species::Cat, 3),
            (Species::Armadillo, 1),
            (Species::Ostrich, 2),
            (Species::Fly, 4),
            (Species::Beetle, 4),
        ],
        Overlay::Jungle => &[
            (Species::Snake, 1),
            (Species::Cat, 2),
            (Species::Monkey, 1),
            (Species::Frog, 3),
            (Species::Scorpion, 1),
        ],
        Overlay::Desert => &[
            (Species::Cat, 1),
            (Species::Snake, 1),
            (Species::Scorpion, 1),
        ],
        Overlay::Lemon => &[(Species::CitrusSnail, 1)],
        Overlay::Frost => &[
            (Species::Penguin, 4),
            (Species::Bear, 1),
            (Species::Wolf, 2),
            (Species::SnowFigure, 1),
        ],
        Overlay::Swamp => &[
            (Species::Crocodile, 1),
            (Species::Snake, 4),
            (Species::Frog, 6),
            (Species::Fly, 5),
            (Species::Turtle, 3),
            (Species::Capybara, 1),
        ],
        Overlay::Crystal => &[(Species::CrystalBlade, 2)],
        Overlay::Enchanted => &[(Species::Butterfly, 5), (Species::Fairy, 1)],
        Overlay::Blight => &[(Species::HoppingSkull, 1)],
        Overlay::Infernal => &[(Species::BurningSkull, 1)],
        Overlay::Candy => &[(Species::ToastyBlob, 1)],
        Overlay::Mushroom => &[(Species::Frog, 3), (Species::Sheep, 1)],
        Overlay::Wasteland => &[(Species::Rat, 4), (Species::AcidBlob, 1)],
        Overlay::Birch => &[
            (Species::Wolf, 1),
            (Species::Sheep, 1),
            (Species::Rabbit, 1),
            (Species::Fox, 1),
        ],
        Overlay::Maple => &[(Species::Wolf, 1), (Species::Dog, 1)],
        Overlay::Rockland => &[
            (Species::Goat, 1),
            (Species::Alpaca, 1),
            (Species::Armadillo, 1),
        ],
        Overlay::Garlic => &[(Species::GarlicSprite, 1)],
        Overlay::Flower => &[
            (Species::BudWalker, 1),
            (Species::Butterfly, 1),
            (Species::Beetle, 1),
            (Species::Bee, 1),
            (Species::Grasshopper, 1),
            (Species::Fly, 1),
        ],
        Overlay::Starlight => &[(Species::Unicorn, 1)],
        Overlay::Clover => &[
            (Species::Rabbit, 1),
            (Species::Cow, 1),
            (Species::Butterfly, 5),
            (Species::Bee, 3),
            (Species::Fairy, 1),
        ],
        Overlay::Paradox => &[(Species::Scorpion, 1)],
        Overlay::Bare | Overlay::Pumpkin | Overlay::Void => &[],
    }
}

pub fn max_random_amount(species: Species) -> usize {
    match species {
        Species::AcidBlob => 10,
        Species::Bee | Species::Beetle | Species::Fly | Species::Grasshopper | Species::Rat => 5,
        Species::Butterfly => 6,
        Species::BurningSkull | Species::HoppingSkull => 4,
        Species::Crocodile | Species::Hyena => 2,
        Species::Rhino => 1,
        _ => 3,
    }
}

fn spread_biome(
    map: &mut TileMap,
    pos: TilePos,
    force: bool,
    randy: &mut Randy,
    landing: &mut Landing,
) {
    let Some(source) = map.get(pos) else { return };

    if source.frozen || source.overlay == Overlay::Bare {
        return;
    }

    let biome = source.overlay;

    for near in neighbours(map, pos) {
        let Some(tile) = map.get(near) else { continue };

        if tile.frozen || tile.wall.is_some() || !tile.ground.is_soil() || tile.overlay == biome {
            continue;
        }

        let take = if tile.overlay == Overlay::Bare || force {
            true
        } else {
            let mine = grow_strength(biome);
            let theirs = grow_strength(tile.overlay);
            if mine == theirs {
                randy.random_chance(SAME_STRENGTH_ODDS)
            } else {
                randy.random_int(0, mine) > randy.random_int(0, theirs)
            }
        };

        if take {
            landing.set(map, near, tile.with_overlay(biome));
        }
    }
}

fn rain_on(map: &mut TileMap, pos: TilePos, randy: &mut Randy, landing: &mut Landing) {
    landing.douse.push(pos);
    for status in [Status::Burning, Status::Angry] {
        landing.relieve.push(Relieve {
            centre: pos,
            reach: 0.0,
            status,
        });
    }

    let Some(tile) = map.get(pos) else { return };

    if tile.is_lava()
        && let Some(cooled) = tile.ground.increase_to()
    {
        landing.set(map, pos, Tile::bare(cooled));
    }

    if let Some(tile) = map.get(pos)
        && spreads_by_water(tile.overlay)
    {
        spread_biome(map, pos, false, randy, landing);
    }
}

fn acid_on(
    map: &mut TileMap,
    frost: &mut Frost,
    pos: TilePos,
    randy: &mut Randy,
    landing: &mut Landing,
) {
    let mut touched = Touched::default();

    if map.get(pos).is_some_and(|tile| tile.frozen) {
        thaw(map, frost, pos, 99, &mut touched);
    }
    landing.take(touched);

    let Some(tile) = map.get(pos) else { return };

    if tile.overlay != Overlay::Wasteland {
        if tile.overlay != Overlay::Bare {
            landing.set(map, pos, tile.with_overlay(Overlay::Bare));
        } else if tile.ground.is_soil() && tile.wall.is_none() {
            landing.set(map, pos, tile.with_overlay(Overlay::Wasteland));
        }
    }

    if randy.random_chance(ACID_SMOKE_ODDS) {
        landing.emit.push(Emit {
            glow: Glow::Smoke,
            pos,
            unless_flashing: false,
        });
    }

    landing.harm.push(Harm {
        centre: pos,
        reach: UNIT_REACH,
        amount: ACID_HURT,
        chance: 1.0 - ACID_SPARE,
    });

    if map
        .get(pos)
        .is_some_and(|tile| tile.overlay == Overlay::Wasteland)
    {
        spread_biome(map, pos, true, randy, landing);
    }
}

fn snow_on(
    map: &mut TileMap,
    frost: &mut Frost,
    pos: TilePos,
    randy: &mut Randy,
    landing: &mut Landing,
) {
    let mut touched = Touched::default();

    if map.get(pos).is_some_and(Tile::can_freeze) {
        freeze(map, frost, pos, &mut touched);
    }

    let x0 = pos.x / CHUNK_SIZE * CHUNK_SIZE;
    let y0 = pos.y / CHUNK_SIZE * CHUNK_SIZE;
    let x1 = (x0 + CHUNK_SIZE).min(map.width);
    let y1 = (y0 + CHUNK_SIZE).min(map.height);

    for _ in 0..SNOW_TRIES {
        let x = randy.random_int(x0 as i32, x1 as i32) as u32;
        let y = randy.random_int(y0 as i32, y1 as i32) as u32;
        let other = TilePos::new(x, y);

        if map.get(other).is_some_and(Tile::can_freeze) {
            let dx = x as f32 - pos.x as f32;
            let dy = y as f32 - pos.y as f32;
            if dx.hypot(dy) < SNOW_SPREAD {
                break;
            }
            freeze(map, frost, other, &mut touched);
        }
    }

    landing.take(touched);

    if map.get(pos).is_some_and(Tile::is_lava) {
        return;
    }

    if randy.random_bool() {
        landing.afflict(pos, Status::Frozen, 1.0);
    }
}

fn seed_life(
    map: &TileMap,
    pos: TilePos,
    census: &Census,
    crowd: usize,
    randy: &mut Randy,
    landing: &mut Landing,
) {
    let Some(tile) = map.get(pos) else { return };
    let pot = spawn_pot(tile);
    let total: u32 = pot.iter().map(|(_, weight)| weight).sum();

    if total == 0 {
        return;
    }

    let mut roll = randy.random_int(0, total as i32) as u32;
    let Some(species) = pot.iter().find_map(|&(species, weight)| {
        if roll < weight {
            Some(species)
        } else {
            roll -= weight;
            None
        }
    }) else {
        return;
    };

    if census.of(species) > max_random_amount(species) || crowd > CROWD {
        return;
    }

    landing.spawn.push(Spawn { species, pos });
}

pub fn land(
    kind: DropKind,
    map: &mut TileMap,
    frost: &mut Frost,
    pos: TilePos,
    census: &Census,
    crowd: usize,
    randy: &mut Randy,
) -> Landing {
    let mut landing = Landing::default();

    match kind {
        DropKind::Rain => {
            rain_on(map, pos, randy, &mut landing);
            for near in neighbours(map, pos) {
                rain_on(map, near, randy, &mut landing);
            }
        }
        DropKind::Acid => acid_on(map, frost, pos, randy, &mut landing),
        DropKind::Fire => {
            if randy.random_chance(FIRE_GLOW_ODDS) {
                landing.emit.push(Emit {
                    glow: Glow::Fire,
                    pos,
                    unless_flashing: true,
                });
            }
            landing.ignite.push(pos);
            landing.afflict(pos, Status::Burning, 1.0);
            if map
                .get(pos)
                .is_some_and(|tile| tile.overlay == Overlay::Infernal)
            {
                spread_biome(map, pos, true, randy, &mut landing);
            }
        }
        DropKind::Lava => {
            if map
                .get(pos)
                .is_some_and(|tile| tile.ground != Ground::Lava3)
            {
                landing.ignite.push(pos);
                landing.set(map, pos, Tile::bare(Ground::Lava3));
            }
        }
        DropKind::Snow => snow_on(map, frost, pos, randy, &mut landing),
        DropKind::Ash => {
            landing.afflict(pos, Status::Cough, 0.3);
            landing.afflict(pos, Status::AshFever, 0.1);
        }
        DropKind::MagicRain => {
            for status in [
                Status::Powerup,
                Status::SpellBoost,
                Status::Shield,
                Status::Caffeinated,
            ] {
                landing.afflict(pos, status, 0.2);
            }
        }
        DropKind::Rage => landing.afflict(pos, Status::Rage, 0.2),
        DropKind::LifeSeed => seed_life(map, pos, census, crowd, randy, &mut landing),
    }

    landing
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(ground: Ground, overlay: Overlay) -> TileMap {
        TileMap::filled(32, 32, Tile::new(ground, overlay))
    }

    fn drop(kind: DropKind, map: &mut TileMap, pos: TilePos) -> Landing {
        land(
            kind,
            map,
            &mut Frost::default(),
            pos,
            &Census::default(),
            0,
            &mut Randy::new(7),
        )
    }

    #[test]
    fn rain_douses_and_soothes_the_tile_and_its_four_neighbours() {
        let mut map = field(Ground::Sand, Overlay::Bare);
        let landing = drop(DropKind::Rain, &mut map, TilePos::new(5, 5));

        assert_eq!(landing.douse.len(), 5);
        assert_eq!(landing.relieve.len(), 10);
        assert!(landing.cells.is_empty());
        assert_eq!(map.at(5, 5), Tile::bare(Ground::Sand));
    }

    #[test]
    fn rain_cools_lava_into_hills() {
        let mut map = field(Ground::Lava3, Overlay::Bare);
        drop(DropKind::Rain, &mut map, TilePos::new(5, 5));

        assert_eq!(map.at(5, 5).ground, Ground::Hills);
        assert_eq!(map.at(6, 5).ground, Ground::Hills);
        assert_eq!(map.at(6, 6).ground, Ground::Lava3);
    }

    #[test]
    fn rain_spreads_a_thirsty_biome_onto_bare_soil() {
        let mut map = field(Ground::SoilLow, Overlay::Bare);
        map.set(5, 5, Tile::new(Ground::SoilLow, Overlay::Grass));
        drop(DropKind::Rain, &mut map, TilePos::new(5, 5));

        for (x, y) in [(4, 5), (6, 5), (5, 4), (5, 6)] {
            assert_eq!(map.at(x, y).overlay, Overlay::Grass);
        }
    }

    #[test]
    fn acid_strips_a_biome_then_leaves_wasteland_on_bare_soil() {
        let mut map = field(Ground::SoilLow, Overlay::Grass);
        drop(DropKind::Acid, &mut map, TilePos::new(5, 5));
        assert_eq!(map.at(5, 5).overlay, Overlay::Bare);

        let landing = drop(DropKind::Acid, &mut map, TilePos::new(5, 5));
        assert_eq!(map.at(5, 5).overlay, Overlay::Wasteland);
        assert_eq!(landing.harm[0].amount, ACID_HURT);
        assert!((landing.harm[0].chance - 0.4).abs() < 1e-6);
    }

    #[test]
    fn acid_spreads_wasteland_by_force_and_melts_ice() {
        let mut map = field(Ground::SoilLow, Overlay::Grass);
        map.set(5, 5, Tile::new(Ground::SoilLow, Overlay::Wasteland));
        drop(DropKind::Acid, &mut map, TilePos::new(5, 5));
        assert_eq!(map.at(6, 5).overlay, Overlay::Wasteland);

        let mut ice = field(Ground::Sand, Overlay::Bare);
        ice.set(2, 2, Tile::bare(Ground::Sand).with_frozen(true));
        drop(DropKind::Acid, &mut ice, TilePos::new(2, 2));
        assert!(!ice.at(2, 2).frozen);
    }

    #[test]
    fn fire_sets_the_ground_alight_and_everyone_near_burning() {
        let mut map = field(Ground::SoilLow, Overlay::Grass);
        let landing = drop(DropKind::Fire, &mut map, TilePos::new(3, 3));

        assert_eq!(landing.ignite, [TilePos::new(3, 3)]);
        assert_eq!(landing.afflict[0].status, Status::Burning);
        assert_eq!(landing.afflict[0].reach, UNIT_REACH);
    }

    #[test]
    fn one_acid_drop_in_five_smokes_and_one_fire_drop_in_five_glows() {
        let mut randy = Randy::new(3);
        let mut count = |kind| {
            let mut emits = Vec::new();
            for _ in 0..1000 {
                let mut map = field(Ground::SoilLow, Overlay::Grass);
                emits.extend(
                    land(
                        kind,
                        &mut map,
                        &mut Frost::default(),
                        TilePos::new(3, 3),
                        &Census::default(),
                        0,
                        &mut randy,
                    )
                    .emit,
                );
            }
            emits
        };

        let smoke = count(DropKind::Acid);
        assert!((120..280).contains(&smoke.len()), "{}", smoke.len());
        assert!(
            smoke
                .iter()
                .all(|emit| emit.glow == Glow::Smoke && !emit.unless_flashing)
        );

        let fire = count(DropKind::Fire);
        assert!((120..280).contains(&fire.len()), "{}", fire.len());
        assert!(
            fire.iter()
                .all(|emit| emit.glow == Glow::Fire && emit.unless_flashing)
        );

        for kind in [
            DropKind::Rain,
            DropKind::Lava,
            DropKind::Snow,
            DropKind::Ash,
        ] {
            assert!(count(kind).is_empty(), "{kind:?}");
        }
    }

    #[test]
    fn lava_fills_any_tile_but_its_own_hottest_kind() {
        let mut map = field(Ground::DeepOcean, Overlay::Bare);
        let landing = drop(DropKind::Lava, &mut map, TilePos::new(3, 3));
        assert_eq!(map.at(3, 3), Tile::bare(Ground::Lava3));
        assert_eq!(landing.ignite.len(), 1);

        let again = drop(DropKind::Lava, &mut map, TilePos::new(3, 3));
        assert!(again.ignite.is_empty() && again.cells.is_empty());
    }

    #[test]
    fn snow_freezes_its_tile_and_sometimes_a_far_corner_of_the_chunk() {
        let mut map = field(Ground::SoilLow, Overlay::Grass);
        let mut frost = Frost::default();
        let mut randy = Randy::new(3);
        let census = Census::default();
        let mut frozen_units = 0;

        for _ in 0..40 {
            let landing = land(
                DropKind::Snow,
                &mut map,
                &mut frost,
                TilePos::new(0, 0),
                &census,
                0,
                &mut randy,
            );
            frozen_units += landing.afflict.len();
        }

        assert!(map.at(0, 0).frozen);
        assert!((5..35).contains(&frozen_units), "{frozen_units}");
        assert!((0..16).all(|y| (0..16).all(|x| {
            let near = ((x * x + y * y) as f32).sqrt() < SNOW_SPREAD;
            !map.at(x, y).frozen || x + y == 0 || !near
        })));
        assert!((16..32).all(|y| (0..32).all(|x| !map.at(x, y).frozen)));
    }

    #[test]
    fn ash_magic_and_rage_afflict_with_worldbox_odds() {
        let mut map = field(Ground::SoilLow, Overlay::Grass);
        let at = TilePos::new(4, 4);
        let odds = |landing: Landing| -> Vec<(Status, f32)> {
            landing
                .afflict
                .iter()
                .map(|a| (a.status, a.chance))
                .collect()
        };

        assert_eq!(
            odds(drop(DropKind::Ash, &mut map, at)),
            [(Status::Cough, 0.3), (Status::AshFever, 0.1)]
        );
        assert_eq!(
            odds(drop(DropKind::MagicRain, &mut map, at)),
            [
                (Status::Powerup, 0.2),
                (Status::SpellBoost, 0.2),
                (Status::Shield, 0.2),
                (Status::Caffeinated, 0.2)
            ]
        );
        assert_eq!(
            odds(drop(DropKind::Rage, &mut map, at)),
            [(Status::Rage, 0.2)]
        );
    }

    #[test]
    fn a_life_seed_brings_an_animal_of_the_biome_unless_the_place_is_crowded() {
        let mut map = field(Ground::SoilLow, Overlay::Frost);
        let landing = drop(DropKind::LifeSeed, &mut map, TilePos::new(4, 4));
        let species = landing.spawn[0].species;
        assert!(
            spawn_pot(map.at(4, 4))
                .iter()
                .any(|(kind, _)| *kind == species)
        );

        let crowded = land(
            DropKind::LifeSeed,
            &mut map,
            &mut Frost::default(),
            TilePos::new(4, 4),
            &Census::default(),
            CROWD + 1,
            &mut Randy::new(7),
        );
        assert!(crowded.spawn.is_empty());

        let mut bare = field(Ground::SoilLow, Overlay::Bare);
        assert!(
            drop(DropKind::LifeSeed, &mut bare, TilePos::new(4, 4))
                .spawn
                .is_empty()
        );
    }

    #[test]
    fn sand_grows_crabs_and_every_biome_animal_is_a_known_species() {
        assert_eq!(spawn_pot(Tile::bare(Ground::Sand)), &[(Species::Crab, 1)]);

        for overlay in econbox_tile::OVERLAYS {
            for (species, weight) in spawn_pot(Tile::new(Ground::SoilHigh, overlay)) {
                assert!(*weight > 0);
                assert!(econbox_creature::SPECIES.contains(species));
            }
        }
    }
}
