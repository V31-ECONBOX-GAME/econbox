use crate::biome::{
    BIOME_SAND, Biome, CACTI_TREE, FRUIT_BUSH, PALM_TREE, Pot, Vegetation, biome_of, pool,
};
use crate::shapes::shape_masks;
use crate::stages::{Stage, stages};
use crate::template::{MapGenTemplate, MapGenValues};
use crate::tool::{self, EdgeMasks, dist_tile, fill_tiles, raycast};
use crate::world::{CHUNK_SIZE, World, ZONE_SIZE};
use econbox_field::Mask;
use econbox_noise::Randy;
use econbox_tile::{Flora, Ground, Rank};

pub const TREES_PER_ZONE_FOR_SPAWN: i32 = 3;

pub struct Generator {
    pub world: World,
    pub randy: Randy,
    template: &'static MapGenTemplate,
    values: MapGenValues,
    shapes: Vec<Mask>,
    edges: EdgeMasks,
    pool: Vec<&'static Biome>,
    planting: Option<Planting>,
    rooms: Vec<Room>,
}

#[derive(Debug, Clone, Default)]
struct Room {
    up: Vec<usize>,
    down: Vec<usize>,
    left: Vec<usize>,
    right: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Growth {
    Plants,
    Trees,
    Bushes,
}

impl Generator {
    pub fn new(
        side: u32,
        seed: u32,
        template: &'static MapGenTemplate,
        values: MapGenValues,
    ) -> Self {
        Self {
            world: World::new(side, side),
            randy: Randy::new(seed as i32),
            template,
            values,
            shapes: shape_masks(),
            edges: EdgeMasks::new(),
            pool: pool(),
            planting: None,
            rooms: Vec::new(),
        }
    }

    pub fn plan(&mut self) -> Vec<Stage> {
        let late_subtract = self.randy.random_bool();

        stages(self.template, &self.values, late_subtract)
    }

    pub fn run_all(mut self) -> World {
        for stage in self.plan() {
            self.run(stage);
        }

        self.world
    }

    pub fn run(&mut self, stage: Stage) {
        match stage {
            Stage::RandomShapes { amount, subtract } => {
                for _ in 0..amount {
                    tool::apply_random_shape(
                        &mut self.world,
                        &mut self.randy,
                        &self.shapes,
                        2.0,
                        0.7,
                        subtract,
                    );
                }
            }
            Stage::PerlinNoise => self.perlin(1.0, self.values.perlin_scale_stage_1 as f32, false),
            Stage::ForceHeight(height) => {
                for index in 0..self.world.cells() {
                    self.world.set_height(index, height);
                }
            }
            Stage::CenterGradient => self.center_gradient(),
            Stage::CenterMountains => self.center_mountains(),
            Stage::CenterLake => self.center_lake(),
            Stage::PerlinNoise2 => {
                self.perlin(0.2, 4.0 * self.values.perlin_scale_stage_2 as f32, true)
            }
            Stage::PerlinNoise3 => {
                self.perlin(0.1, self.values.perlin_scale_stage_3 as f32 * 10.0, true)
            }
            Stage::LowGround => self.low_ground(),
            Stage::HighGround => self.high_ground(),
            Stage::RingEffect => tool::apply_ring_effect(&mut self.world),
            Stage::RoundEdges => tool::add_edge_gradient_circle(&mut self.world),
            Stage::SquareEdges => tool::add_edge_square(&mut self.world, &self.edges),
            Stage::UpdateTileTypes => self.world.update_tile_types(),
            Stage::ForbiddenKnowledge => {}
            Stage::RemoveMountains => self.remove_mountains(),
            Stage::Anthill => self.anthill(),
            Stage::Checkerboard => self.checkerboard(),
            Stage::Cubicles => self.cubicles(),
            Stage::PerlinReplace(option) => {
                let container = self.template.perlin_replace[option];
                tool::apply_perlin_replace(&mut self.world, &mut self.randy, &container);
            }
            Stage::Biomes => self.generate_biomes(),
            Stage::MountainEdges => self.mountain_edges(),
            Stage::FreezeMountainTops => self.freeze_mountain_tops(),
            Stage::Vegetation { rounds } => {
                let spread = self.world.cells() / 20;

                for _ in 0..rounds {
                    self.spawn_vegetation(spread);
                }
            }
            Stage::Resources => self.spawn_resources(self.world.cells() / 1000 / 2 / 2),
        }
    }

    fn perlin(&mut self, alpha: f32, scale: f32, subtract: bool) {
        let x = self.randy.random_int(0, 1_000_000) as f32;
        let y = self.randy.random_int(0, 1_000_000) as f32;

        tool::apply_perlin_noise(&mut self.world, x, y, alpha, scale, subtract);
    }

    fn centre(&self) -> ((i32, i32), f32) {
        (
            (
                (self.world.width / 2) as i32,
                (self.world.height / 2) as i32,
            ),
            (self.world.width / 2) as f32,
        )
    }

    fn distance(&self, index: usize, to: (i32, i32)) -> f32 {
        let (x, y) = self.world.position(index);

        dist_tile((x as i32, y as i32), to)
    }

    fn center_gradient(&mut self) {
        let (centre, half) = self.centre();
        let outer = half * 0.9;
        let span = outer - half * 0.6;

        for index in 0..self.world.cells() {
            let dist = self.distance(index, centre);

            if dist <= outer {
                let lift = (45.0 * ((outer - dist) / span)) as i32;
                self.world.add_height(index, lift);
            }
        }
    }

    fn center_lake(&mut self) {
        let (centre, half) = self.centre();
        let outer = half * 0.6;
        let span = outer - half * 0.2;

        for index in 0..self.world.cells() {
            let dist = self.distance(index, centre);

            if dist <= outer {
                let kept = 1.0 - (outer - dist) / span;
                let height = (self.world.height_at(index) as f32 * kept) as i32;
                self.world.set_height(index, height);
            }
        }
    }

    fn center_mountains(&mut self) {
        let (centre, half) = self.centre();
        let outer = half * 0.3;

        for index in 0..self.world.cells() {
            let dist = self.distance(index, centre);
            let sink = (75.0 * ((outer - dist) / outer)) as i32;
            self.world.add_height(index, -sink);
        }
    }

    fn low_ground(&mut self) {
        for index in 0..self.world.cells() {
            if self.world.height_at(index) > 150 {
                self.world.add_height(index, -50);
            }
            if self.world.height_at(index) > 130 {
                self.world.add_height(index, -20);
            }
        }
    }

    fn high_ground(&mut self) {
        for index in 0..self.world.cells() {
            let height = self.world.height_at(index);

            if height < 20 {
                self.world.add_height(index, 80);
            } else if height < 120 {
                self.world.add_height(index, 40);
            }
        }
    }

    fn rocks(&self, index: usize) -> bool {
        self.world.top[index].is_none() && self.world.main[index].is_rock()
    }

    fn decrease(&mut self, index: usize) {
        if self.world.frozen[index] {
            self.world.frozen[index] = false;
        } else if self.world.top[index].is_some() {
            let main = self.world.main[index];
            self.world.terraform(index, main);
        } else if let Some(lower) = self.world.main[index].decrease_to() {
            self.world.terraform(index, lower);
        }
    }

    fn remove_mountains(&mut self) {
        for index in 0..self.world.cells() {
            for _ in 0..2 {
                if self.rocks(index) {
                    self.decrease(index);
                }
            }
        }
    }

    fn use_seed_on(&mut self, index: usize, biome: &Biome) {
        self.world.frozen[index] = false;

        if !self.world.can_be_biome(index) {
            return;
        }

        if matches!(self.world.main[index].rank(), Rank::Low | Rank::High) {
            self.world.set_top(index, biome.overlay);
        }
    }

    fn chunks(&self) -> (u32, u32) {
        (
            self.world.width / CHUNK_SIZE,
            self.world.height / CHUNK_SIZE,
        )
    }

    fn chunk_tiles(&self, chunk: usize) -> Vec<usize> {
        let (columns, _) = self.chunks();
        let cx = chunk as u32 % columns;
        let cy = chunk as u32 / columns;
        let mut tiles = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize);

        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                tiles.push(self.world.index(cx * CHUNK_SIZE + i, cy * CHUNK_SIZE + j));
            }
        }

        tiles
    }

    fn chunk_step(&self, chunk: usize, dx: i32, dy: i32) -> Option<usize> {
        let (columns, rows) = self.chunks();
        let x = (chunk as u32 % columns) as i32 + dx;
        let y = (chunk as u32 / columns) as i32 + dy;

        (x >= 0 && y >= 0 && (x as u32) < columns && (y as u32) < rows)
            .then(|| (y as u32 * columns + x as u32) as usize)
    }

    fn checkerboard(&mut self) {
        let first = *self.randy.pick(&self.pool);
        let second = *self.randy.pick(&self.pool);
        let (columns, rows) = self.chunks();

        for chunk in 0..(columns * rows) as usize {
            let cx = chunk as u32 % columns;
            let cy = chunk as u32 / columns;
            let (ground, biome) = if (cx + cy).is_multiple_of(2) {
                (Ground::SoilHigh, first)
            } else {
                (Ground::SoilLow, second)
            };

            for index in self.chunk_tiles(chunk) {
                self.world.terraform(index, ground);

                if self.values.random_biomes {
                    self.use_seed_on(index, biome);
                }
            }
        }
    }

    fn cubicles(&mut self) {
        let (columns, rows) = self.chunks();
        let count = (columns * rows) as usize;
        let mut used = vec![false; count];
        let mut left: Vec<usize> = (0..count).collect();
        self.rooms.clear();

        while !left.is_empty() {
            let start = left[self.randy.index(left.len())];
            self.start_cubicle(&mut used, &mut left, start);
        }

        for room in std::mem::take(&mut self.rooms) {
            self.make_door(&room.down);
            self.make_door(&room.left);
            self.make_door(&room.right);
            self.make_door(&room.up);
        }
    }

    fn start_cubicle(&mut self, used: &mut [bool], left: &mut Vec<usize>, start: usize) {
        let mut room = Vec::new();
        let remember =
            |chunk: usize, used: &mut [bool], left: &mut Vec<usize>, room: &mut Vec<usize>| {
                used[chunk] = true;
                if let Some(at) = left.iter().position(|c| *c == chunk) {
                    left.remove(at);
                }
                room.push(chunk);
            };
        remember(start, used, left, &mut room);

        let min = 2;
        let max = self.values.cubicle_size + 2;

        for dx in [1, -1] {
            let mut chunk = Some(start);
            let steps = self.randy.random_int(min, max);

            for _ in 0..steps {
                let Some(current) = chunk else { break };
                chunk = self.chunk_step(current, dx, 0);

                if let Some(next) = chunk
                    && !used[next]
                {
                    remember(next, used, left, &mut room);
                }
            }
        }

        let strip = room.clone();

        for dy in [-1, 1] {
            let mut front = strip.clone();
            let steps = self.randy.random_int(min, max);

            'grow: for _ in 0..steps {
                let mut next = Vec::with_capacity(front.len());

                for chunk in &front {
                    match self.chunk_step(*chunk, 0, dy) {
                        Some(step) if !used[step] => next.push(step),
                        _ => break 'grow,
                    }
                }

                front.clear();
                for step in next {
                    remember(step, used, left, &mut room);
                    front.push(step);
                }
            }
        }

        self.finish_room(&room);
    }

    fn finish_room(&mut self, chunks: &[usize]) {
        let biome = *self.randy.pick(&self.pool);
        let main = if self.randy.random_bool() {
            Ground::SoilLow
        } else {
            Ground::SoilHigh
        };
        let width = self.world.width as i32;
        let height = self.world.height as i32;
        let corners = [
            (0, 0),
            (width - 1, 0),
            (0, height - 1),
            (width - 1, height - 1),
        ];
        let mut nearest: [Option<(usize, f32)>; 4] = [None; 4];

        for chunk in chunks {
            for index in self.chunk_tiles(*chunk) {
                self.world.terraform(index, main);

                if self.values.random_biomes {
                    self.use_seed_on(index, biome);
                }

                for (corner, best) in corners.iter().zip(nearest.iter_mut()) {
                    let dist = self.distance(index, *corner);

                    if best.is_none_or(|(_, closest)| dist < closest) {
                        *best = Some((index, dist));
                    }
                }
            }
        }

        let [Some(a), Some(b), Some(c), Some(d)] = nearest.map(|best| {
            best.map(|(index, _)| {
                let (x, y) = self.world.position(index);
                (x as i32, y as i32)
            })
        }) else {
            return;
        };

        let room = Room {
            up: fill_tiles(&mut self.world, a, b, Ground::Mountains),
            down: fill_tiles(&mut self.world, c, d, Ground::Mountains),
            left: fill_tiles(&mut self.world, a, c, Ground::Mountains),
            right: fill_tiles(&mut self.world, d, b, Ground::Mountains),
        };
        self.rooms.push(room);
    }

    fn make_door(&mut self, tiles: &[usize]) {
        if tiles.is_empty()
            || tiles
                .iter()
                .any(|index| self.world.main[*index] != Ground::Mountains)
        {
            return;
        }

        let door = if tiles.len() > 3 {
            tiles[self.randy.random_int(3, tiles.len() as i32 - 3) as usize]
        } else {
            tiles[tiles.len() / 2]
        };

        self.world.terraform(door, Ground::Hills);

        let around: Vec<usize> = self.world.neighbours_all(door).collect();
        for index in around {
            if self.world.main[index] == Ground::Mountains {
                self.world.terraform(index, Ground::Hills);
            }
        }
    }

    fn zones(&self) -> (i32, i32) {
        (
            (self.world.width / ZONE_SIZE) as i32,
            (self.world.height / ZONE_SIZE) as i32,
        )
    }

    fn zone_on_edge(&self, zone: (i32, i32)) -> bool {
        let (columns, rows) = self.zones();

        zone.0 == 0 || zone.1 == 0 || zone.0 == columns - 1 || zone.1 == rows - 1
    }

    fn zone_tiles(&self, zone: (i32, i32)) -> Vec<usize> {
        let mut tiles = Vec::with_capacity((ZONE_SIZE * ZONE_SIZE) as usize);

        for i in 0..ZONE_SIZE as i32 {
            for j in 0..ZONE_SIZE as i32 {
                if let Some(index) = self
                    .world
                    .tile_at(zone.0 * ZONE_SIZE as i32 + i, zone.1 * ZONE_SIZE as i32 + j)
                {
                    tiles.push(index);
                }
            }
        }

        tiles
    }

    fn zone_centre(&self, zone: (i32, i32)) -> (i32, i32) {
        let half = (ZONE_SIZE / 2) as i32;

        (
            zone.0 * ZONE_SIZE as i32 + half,
            zone.1 * ZONE_SIZE as i32 + half,
        )
    }

    fn random_zone_neighbour(&mut self, zone: (i32, i32)) -> (i32, i32) {
        let (columns, rows) = self.zones();
        let around: Vec<(i32, i32)> = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| (zone.0 + dx, zone.1 + dy))
            .filter(|(x, y)| *x >= 0 && *y >= 0 && *x < columns && *y < rows)
            .collect();

        *self.randy.pick(&around)
    }

    fn random_neighbour(&mut self, index: usize) -> usize {
        let around: Vec<usize> = self.world.neighbours(index).collect();

        *self.randy.pick(&around)
    }

    fn anthill(&mut self) {
        for index in 0..self.world.cells() {
            self.world.terraform(index, Ground::Mountains);
        }

        let (columns, rows) = self.zones();
        let step_x = columns / 10 + 1;
        let step_y = rows / 10 + 1;
        let first = (step_x, step_y);
        let second = (columns - step_x, step_y);
        let third = (columns - step_x, rows - step_y);
        let fourth = (step_x, rows - step_y);
        let middle = (columns / 2, rows / 2);
        let mut rooms = Vec::new();
        let mut tunnels = Vec::new();

        for start in [
            middle, middle, first, first, second, second, third, third, fourth, fourth,
        ] {
            self.make_jail_room(&mut rooms, start);
        }

        for (from, to) in [
            (first, second),
            (fourth, third),
            (first, fourth),
            (third, second),
            (third, first),
        ] {
            self.make_way(from, to, &mut tunnels);
        }

        for zone in rooms {
            self.carve_zone(zone);
        }
        for index in tunnels {
            self.carve_tunnel(index);
        }
    }

    fn make_jail_room(&mut self, rooms: &mut Vec<(i32, i32)>, start: (i32, i32)) {
        let (columns, rows) = self.zones();
        let steps = columns * rows / 10;
        let mut zone = start;

        if self.zone_on_edge(zone) {
            return;
        }

        for _ in 0..steps {
            if self.zone_on_edge(zone) {
                zone = self.random_zone_neighbour(zone);
                continue;
            }

            for index in self.zone_tiles(zone) {
                self.world.terraform(index, Ground::SoilHigh);
            }

            rooms.push(zone);
            zone = self.random_zone_neighbour(zone);
        }
    }

    fn make_way(&mut self, from: (i32, i32), to: (i32, i32), tunnels: &mut Vec<usize>) {
        let line = raycast(
            &self.world,
            self.zone_centre(from),
            self.zone_centre(to),
            0.99,
        );

        for index in line {
            let around: Vec<usize> = self.world.neighbours_all(index).collect();

            for next in around {
                self.world.terraform(next, Ground::SoilHigh);
                tunnels.push(next);
            }
        }
    }

    fn carve_zone(&mut self, zone: (i32, i32)) {
        let tiles = self.zone_tiles(zone);

        for _ in 0..20 {
            let mut current = *self.randy.pick(&tiles);

            for _ in 0..15 {
                let next = self.random_neighbour(current);

                if self.rocks(next) {
                    self.world.terraform(current, Ground::SoilHigh);
                    current = next;
                }
            }
        }
    }

    fn carve_tunnel(&mut self, index: usize) {
        for _ in 0..10 {
            let mut current = self.random_neighbour(index);

            for _ in 0..10 {
                let next = self.random_neighbour(current);

                if self.rocks(next) {
                    self.world.terraform(current, Ground::SoilHigh);
                    current = next;
                }
            }
        }
    }

    fn generate_biomes(&mut self) {
        let islands = self.world.islands();
        let cells = self.world.cells();
        let mut soil: Vec<bool> = (0..cells).map(|index| self.world.is_soil(index)).collect();
        let mut remaining = soil.iter().filter(|member| **member).count();
        let mut list: Vec<usize> = Vec::new();
        let mut stamp = vec![0_u32; cells];
        let mut round = 0_u32;
        let mut refresh = true;

        while remaining > 0 {
            if refresh {
                refresh = false;
                list.clear();
                list.extend((0..cells).filter(|index| soil[*index]));
                self.randy.shuffle(&mut list);
            }

            let Some(&start) = list.last() else { break };
            let biome = *self.randy.pick(&self.pool);
            let steps = biome_steps(islands.size_of(start), biome);

            if steps == 0 {
                list.pop();
                continue;
            }

            round += 1;
            self.make_biome(
                start,
                &mut soil,
                &mut remaining,
                steps,
                biome,
                &mut stamp,
                round,
            );
            refresh = true;
        }
    }

    fn make_biome(
        &mut self,
        start: usize,
        soil: &mut [bool],
        remaining: &mut usize,
        max_steps: u32,
        biome: &'static Biome,
        stamp: &mut [u32],
        round: u32,
    ) {
        let mut placed = 0;
        let mut open = vec![start];
        let mut visited = vec![start];
        stamp[start] = round;

        while !open.is_empty() && placed < max_steps {
            self.randy.shuffle_last(&mut open);
            let Some(index) = open.pop() else { break };

            self.world.set_top(index, biome.overlay);
            if soil[index] {
                soil[index] = false;
                *remaining -= 1;
            }
            placed += 1;

            let around: Vec<usize> = self.world.neighbours_all(index).collect();
            for next in around {
                if stamp[next] != round && self.world.is_soil(next) {
                    stamp[next] = round;
                    open.push(next);
                    visited.push(next);
                }
            }
        }

        if placed <= 10 {
            self.remove_small_biome_patch(&visited, start);
        }
    }

    fn remove_small_biome_patch(&mut self, patch: &[usize], start: usize) {
        let own = self.world.top[start]
            .and_then(biome_of)
            .map(|biome| biome.id);
        let neighbour = self
            .world
            .neighbours_all(start)
            .filter_map(|index| self.world.top[index].and_then(biome_of))
            .find(|biome| !biome.special_biome && Some(biome.id) != own);

        let Some(biome) = neighbour else { return };

        for index in patch {
            let top = self.world.main[*index]
                .is_soil()
                .then_some(biome.overlay)
                .flatten();
            self.world.set_top(*index, top);
        }
    }

    fn mountain_edges(&mut self) {
        let right = self.world.width as i32 - 1;
        let top = self.world.height as i32 - 1;

        fill_tiles(&mut self.world, (0, 0), (right, 0), Ground::Mountains);
        fill_tiles(&mut self.world, (0, top), (right, top), Ground::Mountains);
        fill_tiles(&mut self.world, (0, 0), (0, top), Ground::Mountains);
        fill_tiles(&mut self.world, (right, top), (right, 0), Ground::Mountains);
    }

    fn freeze_mountain_tops(&mut self) {
        for index in 0..self.world.cells() {
            if self.world.main[index] == Ground::Mountains
                && self.world.top[index].is_none()
                && self.world.height_at(index) > 220
            {
                self.world.frozen[index] = true;
            }
        }
    }

    fn take_planting(&mut self) -> Planting {
        self.planting
            .take()
            .unwrap_or_else(|| Planting::new(&self.world))
    }

    fn spawn_vegetation(&mut self, amount: usize) {
        let mut planting = self.take_planting();

        for _ in 0..amount {
            let index = self.randy.index(self.world.cells());

            if !self.world.is_ground(index)
                || planting.count(&self.world, index, Flora::Tree) >= TREES_PER_ZONE_FOR_SPAWN
            {
                continue;
            }

            let Some(biome) = self.world.biome(index) else {
                continue;
            };

            if !biome.grow_vegetation_auto {
                continue;
            }

            let growth = match self.randy.random_int(0, 3) {
                0 => Growth::Plants,
                1 => Growth::Trees,
                _ => Growth::Bushes,
            };

            self.grow(&mut planting, index, biome, growth);
        }

        self.planting = Some(planting);
    }

    fn draw(&mut self, pot: Pot) -> Option<Vegetation> {
        if pot.is_empty() {
            return None;
        }

        pot.get(self.randy.index(pot.len()))
    }

    fn grow(
        &mut self,
        planting: &mut Planting,
        index: usize,
        biome: &'static Biome,
        growth: Growth,
    ) {
        let vegetation = match growth {
            Growth::Plants => self.draw(biome.plants),
            Growth::Trees if biome.id == BIOME_SAND.id => {
                Some(if planting.near_liquid(&self.world, index) {
                    PALM_TREE
                } else {
                    CACTI_TREE
                })
            }
            Growth::Trees => self.draw(biome.trees),
            Growth::Bushes => self.draw(biome.bushes),
        };
        let Some(vegetation) = vegetation else { return };

        if planting.reached_limit(&self.world, index, &vegetation) {
            return;
        }
        if vegetation.chance < self.randy.random() {
            return;
        }

        let replace_tiny = self.randy.random_chance(0.1);
        if planting.can_build(&self.world, index, &vegetation, true, replace_tiny) {
            planting.place(&mut self.world, index, vegetation);
        }
    }

    fn spawn_resources(&mut self, amount: usize) {
        let mut planting = self.take_planting();

        for _ in 0..amount {
            let index = self.randy.index(self.world.cells());

            if !self.world.is_ground(index) {
                continue;
            }

            let replace_tiny = self.randy.random_chance(0.1);
            if planting.can_build(&self.world, index, &FRUIT_BUSH, false, replace_tiny) {
                planting.place(&mut self.world, index, FRUIT_BUSH);
            }
        }

        self.planting = Some(planting);
    }
}

pub fn biome_steps(island_tiles: u32, biome: &Biome) -> u32 {
    let steps = if island_tiles < 400 {
        island_tiles
    } else if island_tiles >= 600 {
        island_tiles / 3
    } else {
        island_tiles / 2
    };

    if biome.generator_max_size != 0 && steps > biome.generator_max_size {
        biome.generator_max_size
    } else {
        steps
    }
}

#[derive(Debug, Clone, Copy)]
struct Placed {
    origin: usize,
    vegetation: Vegetation,
    alive: bool,
}

#[derive(Debug, Clone)]
struct Planting {
    occupant: Vec<Option<usize>>,
    placed: Vec<Placed>,
    counts: Vec<[i32; 3]>,
    liquid: Vec<bool>,
    columns: i32,
    rows: i32,
}

fn slot(flora: Flora) -> usize {
    match flora {
        Flora::Tree | Flora::Sapling => 0,
        Flora::Plant => 1,
        Flora::Bush => 2,
    }
}

fn size(flora: Flora) -> u8 {
    match flora {
        Flora::Plant => 0,
        Flora::Bush => 1,
        Flora::Tree | Flora::Sapling => 2,
    }
}

impl Planting {
    fn new(world: &World) -> Self {
        let columns = (world.width / ZONE_SIZE) as i32;
        let rows = (world.height / ZONE_SIZE) as i32;
        let mut liquid = vec![false; (columns * rows) as usize];

        for index in 0..world.cells() {
            if world.is_liquid(index) {
                let (x, y) = world.position(index);
                liquid[(y / ZONE_SIZE * columns as u32 + x / ZONE_SIZE) as usize] = true;
            }
        }

        Self {
            occupant: vec![None; world.cells()],
            placed: Vec::new(),
            counts: vec![[0; 3]; (columns * rows) as usize],
            liquid,
            columns,
            rows,
        }
    }

    fn zone_of(&self, world: &World, index: usize) -> (i32, i32) {
        let (x, y) = world.position(index);

        ((x / ZONE_SIZE) as i32, (y / ZONE_SIZE) as i32)
    }

    fn zone_index(&self, zone: (i32, i32)) -> usize {
        (zone.1 * self.columns + zone.0) as usize
    }

    fn count(&self, world: &World, index: usize, flora: Flora) -> i32 {
        self.counts[self.zone_index(self.zone_of(world, index))][slot(flora)]
    }

    fn near_liquid(&self, world: &World, index: usize) -> bool {
        let (zx, zy) = self.zone_of(world, index);

        (-1..=1).any(|dy| {
            (-1..=1).any(|dx| {
                let (x, y) = (zx + dx, zy + dy);

                x >= 0
                    && y >= 0
                    && x < self.columns
                    && y < self.rows
                    && self.liquid[self.zone_index((x, y))]
            })
        })
    }

    fn reached_limit(&self, world: &World, index: usize, vegetation: &Vegetation) -> bool {
        let mut limit = vegetation.limit_per_zone;

        if world.main[index].rank() == Rank::Low && vegetation.flora == Flora::Tree {
            limit = (limit as f32 * 0.5) as i32;
            if limit == 0 {
                limit = 1;
            }
        }

        if vegetation.limit_in_radius > 0 {
            let (x, y) = world.position(index);
            let radius = vegetation.limit_in_radius as f32;
            let crowded = self.placed.iter().any(|other| {
                let (ox, oy) = world.position(other.origin);

                other.alive
                    && other.vegetation.id == vegetation.id
                    && dist_tile((x as i32, y as i32), (ox as i32, oy as i32)) <= radius
            });

            if crowded {
                return true;
            }
        }

        self.count(world, index, vegetation.flora) >= limit
    }

    fn footprint(
        &self,
        world: &World,
        index: usize,
        vegetation: &Vegetation,
    ) -> Option<Vec<usize>> {
        let (x, y) = world.position(index);
        let print = vegetation.footprint;
        let left = x as i32 - print.left as i32;
        let bottom = y as i32 - print.bottom as i32;
        let mut tiles = Vec::with_capacity((print.width() * print.height()) as usize);

        for i in 0..print.width() as i32 {
            for j in 0..print.height() as i32 {
                tiles.push(world.tile_at(left + i, bottom + j)?);
            }
        }

        Some(tiles)
    }

    fn can_build(
        &self,
        world: &World,
        index: usize,
        vegetation: &Vegetation,
        same_biome: bool,
        replace_tiny: bool,
    ) -> bool {
        let Some(tiles) = self.footprint(world, index, vegetation) else {
            return false;
        };
        let home = world.biome(index).map(|biome| biome.id);

        tiles.into_iter().all(|tile| {
            if same_biome && world.biome(tile).map(|biome| biome.id) != home {
                return false;
            }

            if let Some(other) = self.occupant[tile].map(|at| self.placed[at].vegetation) {
                if vegetation.flora == Flora::Bush || other.flora == Flora::Bush {
                    return false;
                }

                if size(vegetation.flora) <= size(other.flora) {
                    let swap = replace_tiny
                        && size(other.flora) == 0
                        && size(vegetation.flora) == 0
                        && other.id != vegetation.id;

                    if !swap {
                        return false;
                    }
                }
            }

            !world.is_liquid(tile) && !world.is_block(tile)
        })
    }

    fn remove(&mut self, world: &mut World, at: usize) {
        let placed = self.placed[at];

        if !placed.alive {
            return;
        }

        self.placed[at].alive = false;
        for tile in self
            .footprint(world, placed.origin, &placed.vegetation)
            .unwrap_or_default()
        {
            if self.occupant[tile] == Some(at) {
                self.occupant[tile] = None;
            }
        }

        world.flora[placed.origin] = None;
        let zone = self.zone_index(self.zone_of(world, placed.origin));
        self.counts[zone][slot(placed.vegetation.flora)] -= 1;
    }

    fn place(&mut self, world: &mut World, index: usize, vegetation: Vegetation) {
        let Some(tiles) = self.footprint(world, index, &vegetation) else {
            return;
        };

        for tile in &tiles {
            if let Some(at) = self.occupant[*tile] {
                self.remove(world, at);
            }
        }

        let at = self.placed.len();
        self.placed.push(Placed {
            origin: index,
            vegetation,
            alive: true,
        });

        for tile in tiles {
            self.occupant[tile] = Some(at);
        }

        world.flora[index] = Some(vegetation.flora);
        let zone = self.zone_index(self.zone_of(world, index));
        self.counts[zone][slot(vegetation.flora)] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::BIOMES;
    use crate::template::TemplateId;

    fn generator(id: TemplateId, side: u32, seed: u32) -> Generator {
        let template = id.template();

        Generator::new(side, seed, template, template.values)
    }

    #[test]
    fn a_biome_grows_over_a_share_of_its_island() {
        let grass = &BIOMES[0];
        let candy = BIOMES
            .iter()
            .find(|b| b.id == "biome_candy")
            .expect("candy");

        assert_eq!(biome_steps(250, grass), 250);
        assert_eq!(biome_steps(500, grass), 250);
        assert_eq!(biome_steps(900, grass), 300);
        assert_eq!(biome_steps(900, candy), 30);
    }

    #[test]
    fn the_center_gradient_lifts_the_middle_most() {
        let mut generator = generator(TemplateId::Continent, 64, 1);
        generator.run(Stage::CenterGradient);
        let world = &generator.world;

        assert_eq!(world.height_at(world.index(32, 32)), 135);
        assert!(world.height_at(world.index(32, 20)) > world.height_at(world.index(32, 8)));
        assert_eq!(world.height_at(world.index(0, 0)), 0);
    }

    #[test]
    fn low_and_high_ground_follow_the_worldbox_steps() {
        let mut generator = generator(TemplateId::Continent, 16, 1);
        for (index, height) in [(0, 200), (1, 140), (2, 10), (3, 100), (4, 125)] {
            generator.world.set_height(index, height);
        }

        generator.run(Stage::LowGround);
        let low: Vec<i32> = (0..5).map(|i| generator.world.height_at(i)).collect();
        assert_eq!(low, [130, 120, 10, 100, 125]);

        generator.run(Stage::HighGround);
        let high: Vec<i32> = (0..5).map(|i| generator.world.height_at(i)).collect();
        assert_eq!(high, [130, 120, 90, 140, 125]);
    }

    #[test]
    fn removing_mountains_drops_every_rock_by_two_steps() {
        let mut generator = generator(TemplateId::Toast, 16, 1);
        generator.world.main[0] = Ground::Summit;
        generator.world.main[1] = Ground::Mountains;
        generator.world.main[2] = Ground::Hills;
        generator.world.main[3] = Ground::SoilHigh;

        generator.run(Stage::RemoveMountains);

        assert_eq!(
            generator.world.main[..4],
            [
                Ground::Hills,
                Ground::SoilHigh,
                Ground::SoilHigh,
                Ground::SoilHigh
            ]
        );
    }

    #[test]
    fn only_tall_mountains_freeze() {
        let mut generator = generator(TemplateId::Continent, 16, 1);
        generator.world.main[0] = Ground::Mountains;
        generator.world.set_height(0, 225);
        generator.world.main[1] = Ground::Mountains;
        generator.world.set_height(1, 215);
        generator.world.main[2] = Ground::Summit;
        generator.world.set_height(2, 240);

        generator.run(Stage::FreezeMountainTops);

        assert_eq!(generator.world.frozen[..3], [true, false, false]);
    }

    #[test]
    fn mountain_edges_wall_in_the_whole_map() {
        let mut generator = generator(TemplateId::BoxWorld, 32, 1);
        generator.run(Stage::MountainEdges);
        let world = &generator.world;

        for step in 0..32 {
            for (x, y) in [(step, 0), (0, step), (step, 31), (31, step)] {
                assert_eq!(world.main[world.index(x, y)], Ground::Mountains, "{x},{y}");
            }
        }
        assert_eq!(world.main[world.index(5, 5)], Ground::DeepOcean);
    }

    #[test]
    fn biomes_cover_every_soil_tile_and_leave_sand_alone() {
        let mut generator = generator(TemplateId::Continent, 64, 3);
        for index in 0..generator.world.cells() {
            let (x, _) = generator.world.position(index);
            generator.world.main[index] = match x {
                0..=9 => Ground::Sand,
                10..=40 => Ground::SoilLow,
                _ => Ground::SoilHigh,
            };
        }

        generator.run(Stage::Biomes);
        let world = &generator.world;

        for index in 0..world.cells() {
            assert_eq!(world.top[index].is_some(), world.main[index].is_soil());
        }

        let mut kinds: Vec<_> = world.top.iter().flatten().copied().collect();
        kinds.sort_unstable();
        kinds.dedup();
        assert!(kinds.len() > 1, "{kinds:?}");
    }

    #[test]
    fn vegetation_grows_on_biomes_and_respects_the_zone_limits() {
        let mut generator = generator(TemplateId::Continent, 64, 5);
        for index in 0..generator.world.cells() {
            generator.world.main[index] = Ground::SoilHigh;
            generator.world.top[index] = Some(econbox_tile::Overlay::Grass);
        }

        generator.run(Stage::Vegetation { rounds: 12 });
        generator.run(Stage::Resources);
        let world = &generator.world;
        let trees = world
            .flora
            .iter()
            .filter(|f| **f == Some(Flora::Tree))
            .count();
        let plants = world
            .flora
            .iter()
            .filter(|f| **f == Some(Flora::Plant))
            .count();

        assert!(trees > 0 && plants > 0, "{trees} trees {plants} plants");
        assert!(trees <= 64 * 3, "{trees}");

        let planting = generator.planting.as_ref().expect("planting");
        assert!(
            planting
                .counts
                .iter()
                .all(|zone| zone[0] <= 3 && zone[2] <= 1)
        );
    }

    #[test]
    fn checkerboard_alternates_high_and_low_soil_by_chunk() {
        let mut generator = generator(TemplateId::Checkerboard, 64, 2);
        generator.run(Stage::Checkerboard);
        let world = &generator.world;

        assert_eq!(world.main[world.index(0, 0)], Ground::SoilHigh);
        assert_eq!(world.main[world.index(16, 0)], Ground::SoilLow);
        assert_eq!(world.main[world.index(16, 16)], Ground::SoilHigh);
        assert!(world.top[world.index(0, 0)].is_some());
    }

    #[test]
    fn cubicles_split_the_map_into_walled_rooms_with_doors() {
        let mut generator = generator(TemplateId::Cubicles, 128, 2);
        generator.run(Stage::Cubicles);
        let world = &generator.world;
        let mountains = world
            .main
            .iter()
            .filter(|g| **g == Ground::Mountains)
            .count();
        let hills = world.main.iter().filter(|g| **g == Ground::Hills).count();

        assert!(mountains > 128 * 4, "{mountains}");
        assert!(hills > 0);
        assert!(world.main.iter().all(|g| *g != Ground::DeepOcean));
    }

    #[test]
    fn the_anthill_is_rock_with_carved_chambers() {
        let mut generator = generator(TemplateId::Anthill, 128, 2);
        generator.run(Stage::Anthill);
        let world = &generator.world;
        let open = world
            .main
            .iter()
            .filter(|g| **g == Ground::SoilHigh)
            .count();
        let rock = world
            .main
            .iter()
            .filter(|g| **g == Ground::Mountains)
            .count();

        assert!(open > 1000 && rock > 1000, "{open} open {rock} rock");
        assert_eq!(world.main[world.index(0, 0)], Ground::Mountains);
    }
}
