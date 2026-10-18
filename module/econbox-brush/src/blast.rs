use crate::{Brush, Change, ForcedBrush, PointerOverUi, Tool, UndoStack, cells, many, mix, step};
use bevy::prelude::*;
use econbox_camera::TILE_SIZE;
use econbox_clock::WorldClock;
use econbox_creature::{Creature, Effect, Strike, centre_of, tile_of};
use econbox_cursor::Hovered;
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::{Painted, TileMap, TilePos};

pub const BOMB_GAP: f32 = 0.3;
pub const CHAIN_DELAY: f32 = 0.15;
pub const CHARGE_LAYER: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bomb {
    Tnt,
    Timed,
    Water,
    Landmine,
    Grenade,
    Napalm,
    Atom,
    Void,
    Titan,
}

pub const BOMBS: [Bomb; 9] = [
    Bomb::Tnt,
    Bomb::Timed,
    Bomb::Water,
    Bomb::Landmine,
    Bomb::Grenade,
    Bomb::Napalm,
    Bomb::Atom,
    Bomb::Void,
    Bomb::Titan,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fuse {
    Impact,
    Spark,
    Timer(f32),
    Step,
}

impl Bomb {
    pub fn name(self) -> &'static str {
        match self {
            Self::Tnt => "tnt",
            Self::Timed => "tnt_timed",
            Self::Water => "water_bomb",
            Self::Landmine => "landmine",
            Self::Grenade => "grenade",
            Self::Napalm => "napalm_bomb",
            Self::Atom => "atomic_bomb",
            Self::Void => "void_bomb",
            Self::Titan => "titan_bomb",
        }
    }

    pub fn radius(self) -> u32 {
        match self {
            Self::Landmine => 3,
            Self::Grenade => 5,
            Self::Timed => 8,
            Self::Water => 9,
            Self::Tnt => 10,
            Self::Void => 11,
            Self::Napalm => 12,
            Self::Atom => 30,
            Self::Titan => 70,
        }
    }

    pub fn depth(self) -> i32 {
        match self {
            Self::Water | Self::Napalm | Self::Void => 0,
            Self::Grenade | Self::Landmine => 1,
            Self::Tnt | Self::Timed | Self::Atom => 2,
            Self::Titan => 4,
        }
    }

    pub fn fuse(self) -> Fuse {
        match self {
            Self::Tnt | Self::Water => Fuse::Spark,
            Self::Timed => Fuse::Timer(5.0),
            Self::Landmine => Fuse::Step,
            _ => Fuse::Impact,
        }
    }

    pub fn waits(self) -> bool {
        self.fuse() != Fuse::Impact
    }

    pub fn effect(self) -> Effect {
        match self {
            Self::Water => Effect::Hurt(15.0),
            Self::Grenade => Effect::Hurt(50.0),
            Self::Napalm => Effect::Hurt(60.0),
            Self::Landmine => Effect::Hurt(80.0),
            Self::Tnt | Self::Timed => Effect::Hurt(100.0),
            Self::Atom | Self::Void | Self::Titan => Effect::Kill,
        }
    }

    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Tnt => [0xa3, 0x00, 0x00],
            Self::Timed => [0x7f, 0x00, 0x00],
            Self::Water => [0x6d, 0x00, 0xcd],
            Self::Landmine => [0x99, 0x00, 0x00],
            Self::Grenade => [0x4a, 0x5a, 0x2c],
            Self::Napalm => [0xf4, 0x7c, 0x2c],
            Self::Atom => [0xd8, 0xc8, 0x3c],
            Self::Void => [0x2a, 0x12, 0x3e],
            Self::Titan => [0x3c, 0x3c, 0x44],
        }
    }

    fn sparked_by(self, tile: Tile) -> bool {
        tile.is_lava() || (self == Self::Water && tile.is_water())
    }
}

fn scorched(tile: Tile) -> Tile {
    if tile.ground.takes_overlay() {
        Tile::new(tile.ground, Overlay::Infernal)
    } else {
        tile.with_flora(None).with_wall(None)
    }
}

pub fn blasted(bomb: Bomb, tile: Tile, rim: f32, noise: u32) -> Option<Tile> {
    let rim = rim + (noise % 100) as f32 * 0.0016 - 0.08;

    if rim >= 1.0 {
        return None;
    }

    let rim = rim.max(0.0);
    let ground = tile.ground;
    let dig = ((bomb.depth() as f32 * (1.0 - rim) * 1.6) as i32).min(bomb.depth());

    let next = match bomb {
        Bomb::Void => Tile::bare(Ground::DeepOcean),
        Bomb::Water if ground.is_lava() => Tile::bare(Ground::Hills),
        Bomb::Water if rim < 0.6 && ground.takes_overlay() => Tile::bare(Ground::ShallowWaters),
        Bomb::Water => return None,
        Bomb::Napalm if rim < 0.2 && ground.is_land() => Tile::bare(ground),
        _ if dig > 0 => Tile::bare(step(ground, -dig)),
        _ if ground.is_land() => scorched(tile),
        _ => return None,
    };

    (next != tile).then_some(next)
}

pub fn burst(map: &TileMap, bomb: Bomb, centre: TilePos, salt: u32) -> Vec<Change> {
    let reach = bomb.radius() as i32;
    let mut out = Vec::new();

    for dy in -reach..=reach {
        for dx in -reach..=reach {
            if dx * dx + dy * dy > reach * reach + reach {
                continue;
            }

            let (Ok(x), Ok(y)) = (
                u32::try_from(centre.x as i32 + dx),
                u32::try_from(centre.y as i32 + dy),
            ) else {
                continue;
            };
            let pos = TilePos::new(x, y);
            let Some(tile) = map.get(pos) else { continue };
            let rim = (dx as f32).hypot(dy as f32) / reach.max(1) as f32;

            if let Some(next) = blasted(bomb, tile, rim, mix(x, y, salt)) {
                out.push((pos, tile, next));
            }
        }
    }

    out
}

pub fn drops(radius: u32) -> usize {
    1 + radius as usize / 4
}

pub fn scatter(map: &TileMap, brush: &Brush, centre: TilePos, salt: u32) -> Vec<TilePos> {
    let mut spots = cells(map, centre, brush.shape, brush.radius);
    spots.sort_by_key(|pos| mix(pos.x, pos.y, salt));
    spots.truncate(drops(brush.radius));
    spots
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct BombDrop {
    pub bomb: Bomb,
    pub pos: TilePos,
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Detonate {
    pub bomb: Bomb,
    pub pos: TilePos,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Charge {
    pub bomb: Bomb,
    pub pos: TilePos,
    pub wait: Option<f32>,
}

impl Charge {
    pub fn new(bomb: Bomb, pos: TilePos) -> Self {
        Self {
            bomb,
            pos,
            wait: match bomb.fuse() {
                Fuse::Timer(wait) => Some(wait),
                _ => None,
            },
        }
    }

    pub fn light(&mut self) {
        self.wait = Some(self.wait.map_or(CHAIN_DELAY, |wait| wait.min(CHAIN_DELAY)));
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BombRest(pub f32);

pub(crate) fn bombard(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut armed: Local<bool>,
    mut salt: Local<u32>,
    hovered: Res<Hovered>,
    brush: Res<Brush>,
    forced: Option<Res<ForcedBrush>>,
    over_ui: Res<PointerOverUi>,
    time: Res<Time>,
    map: Res<TileMap>,
    mut rest: ResMut<BombRest>,
    mut undo: ResMut<UndoStack>,
    mut dropped: MessageWriter<BombDrop>,
) {
    let Tool::Drop(bomb) = brush.tool else {
        return;
    };

    let fresh = buttons.just_pressed(MouseButton::Left);

    if fresh {
        rest.0 = 0.0;
        *armed = !over_ui.0;
    }

    rest.0 = (rest.0 - time.delta_secs()).max(0.0);

    if over_ui.0 || !*armed || !buttons.pressed(MouseButton::Left) || rest.0 > 0.0 {
        return;
    }

    if bomb.waits() && !fresh && !many(keys) {
        return;
    }

    let Some(centre) = hovered.pos else { return };
    let brush = forced.map_or(*brush, |forced| forced.over(*brush));
    *salt = salt.wrapping_add(1);
    rest.0 = BOMB_GAP;
    undo.begin();

    for pos in scatter(&map, &brush, centre, *salt) {
        dropped.write(BombDrop { bomb, pos });
    }
}

pub(crate) fn land(
    mut commands: Commands,
    mut asked: MessageReader<BombDrop>,
    map: Res<TileMap>,
    charges: Query<&Charge>,
    mut boom: MessageWriter<Detonate>,
) {
    let mut taken: Vec<TilePos> = charges.iter().map(|charge| charge.pos).collect();

    for &BombDrop { bomb, pos } in asked.read() {
        let Some(tile) = map.get(pos) else { continue };

        if !bomb.waits() || bomb.sparked_by(tile) {
            boom.write(Detonate { bomb, pos });
            continue;
        }

        if tile.is_water() || tile.blocks() || taken.contains(&pos) {
            continue;
        }

        taken.push(pos);
        let at = centre_of(&map, pos);
        let [r, g, b] = bomb.color();

        commands.spawn((
            Charge::new(bomb, pos),
            Sprite {
                color: Color::srgb_u8(r, g, b),
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.75)),
                ..default()
            },
            Transform::from_xyz(at.x, at.y, CHARGE_LAYER),
        ));
    }
}

pub(crate) fn tick(
    mut commands: Commands,
    time: Res<Time>,
    clock: Option<Res<WorldClock>>,
    map: Res<TileMap>,
    walkers: Query<(&Creature, &Transform)>,
    mut charges: Query<(Entity, &mut Charge)>,
    mut boom: MessageWriter<Detonate>,
) {
    let step = time.delta_secs() * clock.map_or(1.0, |clock| clock.rate());

    for (entity, mut charge) in &mut charges {
        let Some(tile) = map.get(charge.pos) else {
            commands.entity(entity).despawn();
            continue;
        };

        if charge.bomb.sparked_by(tile) {
            charge.light();
        }

        let stepped = charge.bomb.fuse() == Fuse::Step
            && walkers.iter().any(|(creature, transform)| {
                !creature.species.flies()
                    && tile_of(&map, transform.translation.truncate()) == Some(charge.pos)
            });

        if let Some(wait) = &mut charge.wait {
            *wait -= step;
        }

        if stepped || charge.wait.is_some_and(|wait| wait <= 0.0) {
            boom.write(Detonate {
                bomb: charge.bomb,
                pos: charge.pos,
            });
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn explode(
    mut asked: MessageReader<Detonate>,
    mut salt: Local<u32>,
    mut map: ResMut<TileMap>,
    mut undo: ResMut<UndoStack>,
    mut charges: Query<&mut Charge>,
    mut painted: MessageWriter<Painted>,
    mut strike: MessageWriter<Strike>,
) {
    let mut changed = Vec::new();

    for &Detonate { bomb, pos } in asked.read() {
        *salt = salt.wrapping_add(1);

        for (cell, tile, next) in burst(&map, bomb, pos, *salt) {
            undo.record(cell, tile);
            map.set(cell.x, cell.y, next);
            changed.push(cell);
        }

        strike.write(Strike {
            centre: pos,
            reach: bomb.radius() as f32,
            effect: bomb.effect(),
        });

        let reach = bomb.radius() as f32;

        for mut charge in &mut charges {
            let dx = charge.pos.x as f32 - pos.x as f32;
            let dy = charge.pos.y as f32 - pos.y as f32;

            if dx.hypot(dy) <= reach {
                charge.light();
            }
        }
    }

    if !changed.is_empty() {
        painted.write(Painted::new(changed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shape;

    fn meadow() -> TileMap {
        TileMap::filled(96, 96, Tile::new(Ground::SoilLow, Overlay::Grass))
    }

    fn hit(map: &TileMap, bomb: Bomb, centre: TilePos) -> TileMap {
        let mut after = map.clone();

        for (pos, _, next) in burst(map, bomb, centre, 1) {
            after.set(pos.x, pos.y, next);
        }

        after
    }

    #[test]
    fn every_bomb_has_its_own_name_and_the_toolbar_ids() {
        let mut names: Vec<&str> = BOMBS.iter().map(|bomb| bomb.name()).collect();
        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), BOMBS.len());
        assert_eq!(Bomb::Atom.name(), "atomic_bomb");
        assert_eq!(Bomb::Napalm.name(), "napalm_bomb");
    }

    #[test]
    fn the_titan_bomb_is_the_largest_and_the_landmine_the_smallest() {
        let largest = BOMBS.iter().max_by_key(|bomb| bomb.radius()).copied();
        let smallest = BOMBS.iter().min_by_key(|bomb| bomb.radius()).copied();

        assert_eq!(largest, Some(Bomb::Titan));
        assert_eq!(smallest, Some(Bomb::Landmine));
        assert!(Bomb::Atom.radius() > Bomb::Tnt.radius());
        assert!(Bomb::Tnt.radius() > Bomb::Grenade.radius());
    }

    #[test]
    fn only_the_charges_wait_for_their_moment() {
        let waiting: Vec<Bomb> = BOMBS.into_iter().filter(|bomb| bomb.waits()).collect();

        assert_eq!(
            waiting,
            [Bomb::Tnt, Bomb::Timed, Bomb::Water, Bomb::Landmine]
        );
        assert_eq!(Bomb::Timed.fuse(), Fuse::Timer(5.0));
        assert_eq!(Bomb::Landmine.fuse(), Fuse::Step);
        assert_eq!(Charge::new(Bomb::Timed, TilePos::new(0, 0)).wait, Some(5.0));
        assert_eq!(Charge::new(Bomb::Tnt, TilePos::new(0, 0)).wait, None);
    }

    #[test]
    fn a_lit_charge_goes_off_soon_but_never_later_than_it_would_have() {
        let mut tnt = Charge::new(Bomb::Tnt, TilePos::new(0, 0));
        tnt.light();

        assert_eq!(tnt.wait, Some(CHAIN_DELAY));

        let mut timed = Charge::new(Bomb::Timed, TilePos::new(0, 0));
        timed.wait = Some(0.05);
        timed.light();

        assert_eq!(timed.wait, Some(0.05));
    }

    #[test]
    fn a_crater_is_deepest_at_the_centre_and_scorched_at_the_rim() {
        let centre = TilePos::new(48, 48);
        let after = hit(&meadow(), Bomb::Tnt, centre);
        let core = after.get(centre).unwrap();
        let rim = after.get(TilePos::new(48 + 8, 48)).unwrap();
        let outside = after.get(TilePos::new(48 + 12, 48)).unwrap();

        assert_eq!(core, Tile::bare(Ground::ShallowWaters));
        assert_eq!(rim, Tile::new(Ground::SoilLow, Overlay::Infernal));
        assert_eq!(outside, Tile::new(Ground::SoilLow, Overlay::Grass));
    }

    #[test]
    fn a_blast_knocks_walls_and_trees_down() {
        let centre = TilePos::new(48, 48);
        let mut map = meadow();
        let wall = TilePos::new(51, 48);
        map.set(
            wall.x,
            wall.y,
            map.at(wall.x, wall.y)
                .with_wall(Some(econbox_tile::Wall::Order)),
        );
        let after = hit(&map, Bomb::Grenade, centre);

        assert_eq!(after.get(wall).unwrap().wall, None);
    }

    #[test]
    fn bigger_bombs_reach_further() {
        let centre = TilePos::new(48, 48);
        let map = meadow();
        let reach = |bomb| burst(&map, bomb, centre, 1).len();

        assert!(reach(Bomb::Landmine) < reach(Bomb::Grenade));
        assert!(reach(Bomb::Grenade) < reach(Bomb::Tnt));
        assert!(reach(Bomb::Tnt) < reach(Bomb::Atom));
        assert!(reach(Bomb::Atom) < reach(Bomb::Titan));
    }

    #[test]
    fn the_titan_bomb_digs_down_to_the_deep_sea() {
        let centre = TilePos::new(48, 48);
        let after = hit(&meadow(), Bomb::Titan, centre);

        assert_eq!(after.get(centre).unwrap().ground, Ground::DeepOcean);
    }

    #[test]
    fn a_void_bomb_leaves_only_open_sea() {
        let centre = TilePos::new(48, 48);
        let after = hit(&meadow(), Bomb::Void, centre);

        for dx in 0..=8 {
            assert_eq!(
                after.get(TilePos::new(48 + dx, 48)).unwrap(),
                Tile::bare(Ground::DeepOcean),
                "{dx}"
            );
        }
    }

    #[test]
    fn a_water_bomb_floods_the_lowlands_and_cools_lava() {
        let soil = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert_eq!(
            blasted(Bomb::Water, soil, 0.0, 50),
            Some(Tile::bare(Ground::ShallowWaters))
        );
        assert_eq!(blasted(Bomb::Water, soil, 0.9, 50), None);
        assert_eq!(
            blasted(Bomb::Water, Tile::bare(Ground::Lava3), 0.9, 50),
            Some(Tile::bare(Ground::Hills))
        );
        assert_eq!(
            blasted(Bomb::Water, Tile::bare(Ground::Mountains), 0.0, 50),
            None
        );
    }

    #[test]
    fn a_fire_bomb_burns_wide_but_does_not_dig() {
        let soil = Tile::new(Ground::SoilHigh, Overlay::Jungle);

        assert_eq!(
            blasted(Bomb::Napalm, soil, 0.0, 50),
            Some(Tile::bare(Ground::SoilHigh))
        );
        assert_eq!(
            blasted(Bomb::Napalm, soil, 0.7, 50),
            Some(Tile::new(Ground::SoilHigh, Overlay::Infernal))
        );
    }

    #[test]
    fn the_edge_of_a_blast_is_ragged() {
        let soil = Tile::new(Ground::SoilLow, Overlay::Grass);
        let edge: Vec<bool> = (0..100)
            .map(|noise| blasted(Bomb::Tnt, soil, 0.97, noise).is_some())
            .collect();

        assert!(edge.contains(&true));
        assert!(edge.contains(&false));
    }

    #[test]
    fn a_wider_brush_scatters_more_bombs_inside_itself() {
        let map = meadow();
        let mut brush = Brush::default();
        let centre = TilePos::new(48, 48);
        brush.shape = Shape::Round;
        brush.radius = 0;

        assert_eq!(scatter(&map, &brush, centre, 3), [centre]);

        brush.radius = 12;
        let spots = scatter(&map, &brush, centre, 3);

        assert_eq!(spots.len(), drops(12));
        assert!(spots.len() > 1);
        assert!(
            spots
                .iter()
                .all(|pos| cells(&map, centre, Shape::Round, 12).contains(pos))
        );
        assert_ne!(spots, scatter(&map, &brush, centre, 4));
    }
}
