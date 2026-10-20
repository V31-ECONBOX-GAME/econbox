use crate::template::{MapGenTemplate, MapGenValues};

pub const VEGETATION_ROUNDS: u32 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    RandomShapes { amount: i32, subtract: bool },
    PerlinNoise,
    ForceHeight(i32),
    CenterGradient,
    CenterMountains,
    CenterLake,
    PerlinNoise2,
    PerlinNoise3,
    LowGround,
    HighGround,
    RingEffect,
    RoundEdges,
    SquareEdges,
    UpdateTileTypes,
    ForbiddenKnowledge,
    RemoveMountains,
    Anthill,
    Checkerboard,
    Cubicles,
    PerlinReplace(usize),
    Biomes,
    MountainEdges,
    FreezeMountainTops,
    Vegetation { rounds: u32 },
    Resources,
}

fn random_shapes(values: &MapGenValues, subtract: bool) -> Option<Stage> {
    (values.random_shapes_amount != 0).then_some(Stage::RandomShapes {
        amount: values.random_shapes_amount,
        subtract,
    })
}

pub fn stages(template: &MapGenTemplate, values: &MapGenValues, late_subtract: bool) -> Vec<Stage> {
    let picks = [
        random_shapes(values, true),
        (values.main_perlin_noise_stage && values.perlin_scale_stage_1 > 0)
            .then_some(Stage::PerlinNoise),
        (template.force_height_to > 0).then_some(Stage::ForceHeight(template.force_height_to)),
        values
            .add_center_gradient_land
            .then_some(Stage::CenterGradient),
        values
            .center_gradient_mountains
            .then_some(Stage::CenterMountains),
        values.add_center_lake.then_some(Stage::CenterLake),
        (values.perlin_noise_stage_2 && values.perlin_scale_stage_2 > 0)
            .then_some(Stage::PerlinNoise2),
        (values.perlin_noise_stage_3 && values.perlin_scale_stage_3 > 0)
            .then_some(Stage::PerlinNoise3),
        values.low_ground.then_some(Stage::LowGround),
        values.high_ground.then_some(Stage::HighGround),
        random_shapes(values, true),
        values.ring_effect.then_some(Stage::RingEffect),
        values.gradient_round_edges.then_some(Stage::RoundEdges),
        values.square_edges.then_some(Stage::SquareEdges),
        Some(Stage::UpdateTileTypes),
        values
            .forbidden_knowledge_start
            .then_some(Stage::ForbiddenKnowledge),
        values.remove_mountains.then_some(Stage::RemoveMountains),
        template.special_anthill.then_some(Stage::Anthill),
        template.special_checkerboard.then_some(Stage::Checkerboard),
        template.special_cubicles.then_some(Stage::Cubicles),
        random_shapes(values, late_subtract),
    ];
    let late = [
        values.random_biomes.then_some(Stage::Biomes),
        values.add_mountain_edges.then_some(Stage::MountainEdges),
        template
            .freeze_mountains
            .then_some(Stage::FreezeMountainTops),
        values.add_vegetation.then_some(Stage::Vegetation {
            rounds: VEGETATION_ROUNDS,
        }),
        values.add_resources.then_some(Stage::Resources),
    ];

    picks
        .into_iter()
        .flatten()
        .chain((0..template.perlin_replace.len()).map(Stage::PerlinReplace))
        .chain(late.into_iter().flatten())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TemplateId;

    fn plan(id: TemplateId) -> Vec<Stage> {
        let template = id.template();

        stages(template, &template.values, true)
    }

    #[test]
    fn the_continent_runs_the_worldbox_stages_in_order() {
        let shapes = Stage::RandomShapes {
            amount: 5,
            subtract: true,
        };

        assert_eq!(
            plan(TemplateId::Continent),
            [
                shapes,
                Stage::PerlinNoise,
                Stage::CenterGradient,
                Stage::PerlinNoise2,
                Stage::PerlinNoise3,
                shapes,
                Stage::RingEffect,
                Stage::RoundEdges,
                Stage::UpdateTileTypes,
                shapes,
                Stage::PerlinReplace(0),
                Stage::Biomes,
                Stage::FreezeMountainTops,
                Stage::Vegetation { rounds: 12 },
                Stage::Resources,
            ]
        );
    }

    #[test]
    fn the_late_shapes_may_add_land_instead_of_carving_it() {
        let template = TemplateId::Continent.template();
        let plan = stages(template, &template.values, false);

        assert_eq!(
            plan[9],
            Stage::RandomShapes {
                amount: 5,
                subtract: false
            }
        );
    }

    #[test]
    fn the_empty_template_only_turns_heights_into_tiles() {
        assert_eq!(
            plan(TemplateId::Empty),
            [
                Stage::UpdateTileTypes,
                Stage::Biomes,
                Stage::Vegetation { rounds: 12 },
                Stage::Resources,
            ]
        );
    }

    #[test]
    fn each_template_enables_its_own_special_stages() {
        let has = |id: TemplateId, stage: Stage| plan(id).contains(&stage);

        assert!(has(TemplateId::BoxWorld, Stage::MountainEdges));
        assert!(!has(TemplateId::BoxWorld, Stage::RoundEdges));
        assert!(has(TemplateId::Toast, Stage::SquareEdges));
        assert!(has(TemplateId::Toast, Stage::RemoveMountains));
        assert!(!has(TemplateId::Toast, Stage::FreezeMountainTops));
        assert!(has(TemplateId::DormantVolcano, Stage::ForceHeight(125)));
        assert!(!has(TemplateId::DormantVolcano, Stage::Biomes));
        assert!(has(TemplateId::Donut, Stage::CenterLake));
        assert!(has(TemplateId::BadApple, Stage::CenterMountains));
        assert!(has(TemplateId::Lasagna, Stage::LowGround));
        assert!(has(TemplateId::Checkerboard, Stage::Checkerboard));
        assert!(has(TemplateId::Cubicles, Stage::Cubicles));
        assert!(has(TemplateId::Anthill, Stage::Anthill));
        assert!(has(TemplateId::Anthill, Stage::UpdateTileTypes));
        assert!(!has(TemplateId::Islands, Stage::CenterGradient));
        assert_eq!(
            plan(TemplateId::Cheese)
                .iter()
                .filter(|stage| matches!(stage, Stage::PerlinReplace(_)))
                .count(),
            1
        );
    }

    #[test]
    fn a_zero_scale_switches_its_perlin_stage_off() {
        let template = TemplateId::Continent.template();
        let values = MapGenValues {
            perlin_scale_stage_1: 0,
            perlin_scale_stage_3: 0,
            random_shapes_amount: 0,
            ..template.values
        };
        let plan = stages(template, &values, true);

        assert!(!plan.contains(&Stage::PerlinNoise));
        assert!(plan.contains(&Stage::PerlinNoise2));
        assert!(!plan.contains(&Stage::PerlinNoise3));
        assert!(
            !plan
                .iter()
                .any(|stage| matches!(stage, Stage::RandomShapes { .. }))
        );
    }
}
