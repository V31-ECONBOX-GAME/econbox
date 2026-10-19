use econbox_worldgen::{MapSize, TemplateId, WorldConfig, generate};
use std::io::Write;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().unwrap_or_else(|| "world.ppm".into());
    let seed: u32 = args.next().and_then(|v| v.parse().ok()).unwrap_or(1);
    let template = args
        .next()
        .and_then(|id| TemplateId::from_id(&id))
        .unwrap_or_default();
    let size = args
        .next()
        .and_then(|id| MapSize::from_id(&id))
        .unwrap_or_default();

    let started = std::time::Instant::now();
    let map = generate(
        WorldConfig::with_seed(seed)
            .with_template(template)
            .with_size(size),
    );
    let took = started.elapsed();
    let mut out = Vec::new();
    write!(out, "P6\n{} {}\n255\n", map.width, map.height).unwrap();

    for y in 0..map.height {
        for x in 0..map.width {
            out.extend_from_slice(&map.at(x, y).color());
        }
    }

    std::fs::write(&path, out).unwrap();
    println!(
        "{path} {} {}x{} water={:.1}% in {took:?}",
        template.id(),
        map.width,
        map.height,
        map.water_share() * 100.0
    );

    let cells = map.cells() as f32;
    let mut rows: Vec<_> = map
        .tiles
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut acc, tile| {
            *acc.entry(tile.name()).or_insert(0_usize) += 1;
            acc
        })
        .into_iter()
        .collect();
    rows.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    for (name, count) in rows {
        println!("  {name:30} {:5.2}%", count as f32 * 100.0 / cells);
    }

    let flora = map.tiles.iter().filter(|tile| tile.flora.is_some()).count();
    println!("  flora {flora}");
}
