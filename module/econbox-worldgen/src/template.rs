use econbox_tile::Ground;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum TemplateId {
    #[default]
    Continent,
    BoxWorld,
    Islands,
    Toast,
    Pancake,
    BoringPlains,
    Checkerboard,
    Cubicles,
    DormantVolcano,
    Cheese,
    BadApple,
    Donut,
    Lasagna,
    ChaosPearl,
    Anthill,
    Empty,
}

pub const TEMPLATE_IDS: [TemplateId; 16] = [
    TemplateId::Continent,
    TemplateId::BoxWorld,
    TemplateId::Islands,
    TemplateId::Toast,
    TemplateId::Pancake,
    TemplateId::BoringPlains,
    TemplateId::Checkerboard,
    TemplateId::Cubicles,
    TemplateId::DormantVolcano,
    TemplateId::Cheese,
    TemplateId::BadApple,
    TemplateId::Donut,
    TemplateId::Lasagna,
    TemplateId::ChaosPearl,
    TemplateId::Anthill,
    TemplateId::Empty,
];

impl TemplateId {
    pub fn id(self) -> &'static str {
        match self {
            Self::Continent => "continent",
            Self::BoxWorld => "box_world",
            Self::Islands => "islands",
            Self::Toast => "toast",
            Self::Pancake => "pancake",
            Self::BoringPlains => "boring_plains",
            Self::Checkerboard => "checkerboard",
            Self::Cubicles => "cubicles",
            Self::DormantVolcano => "dormant_volcano",
            Self::Cheese => "cheese",
            Self::BadApple => "bad_apple",
            Self::Donut => "donut",
            Self::Lasagna => "lasagna",
            Self::ChaosPearl => "chaos_pearl",
            Self::Anthill => "anthill",
            Self::Empty => "empty",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        TEMPLATE_IDS
            .into_iter()
            .find(|template| template.id() == id)
    }

    pub fn template(self) -> &'static MapGenTemplate {
        &TEMPLATES[self as usize]
    }

    pub fn locale_id(self) -> String {
        format!("template_{}", self.id())
    }

    pub fn description_id(self) -> String {
        format!("template_{}_info", self.id())
    }

    pub fn path_icon(self) -> String {
        format!("ui/new_world_templates_icons/template_{}", self.id())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapGenValues {
    pub perlin_scale_stage_1: i32,
    pub perlin_scale_stage_2: i32,
    pub perlin_scale_stage_3: i32,
    pub main_perlin_noise_stage: bool,
    pub perlin_noise_stage_2: bool,
    pub perlin_noise_stage_3: bool,
    pub square_edges: bool,
    pub gradient_round_edges: bool,
    pub add_center_gradient_land: bool,
    pub center_gradient_mountains: bool,
    pub add_center_lake: bool,
    pub ring_effect: bool,
    pub add_vegetation: bool,
    pub add_resources: bool,
    pub add_mountain_edges: bool,
    pub random_biomes: bool,
    pub random_shapes_amount: i32,
    pub cubicle_size: i32,
    pub remove_mountains: bool,
    pub forbidden_knowledge_start: bool,
    pub low_ground: bool,
    pub high_ground: bool,
}

impl MapGenValues {
    pub const BLANK: Self = Self {
        perlin_scale_stage_1: 5,
        perlin_scale_stage_2: 5,
        perlin_scale_stage_3: 5,
        main_perlin_noise_stage: false,
        perlin_noise_stage_2: false,
        perlin_noise_stage_3: false,
        square_edges: false,
        gradient_round_edges: false,
        add_center_gradient_land: false,
        center_gradient_mountains: false,
        add_center_lake: false,
        ring_effect: false,
        add_vegetation: true,
        add_resources: true,
        add_mountain_edges: false,
        random_biomes: true,
        random_shapes_amount: 0,
        cubicle_size: 0,
        remove_mountains: false,
        forbidden_knowledge_start: false,
        low_ground: false,
        high_ground: false,
    };
}

impl Default for MapGenValues {
    fn default() -> Self {
        TemplateId::default().template().values
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerlinReplaceOption {
    pub replace_height_value: i32,
    pub from: Ground,
    pub to: Ground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerlinReplaceContainer {
    pub scale: i32,
    pub options: &'static [PerlinReplaceOption],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowEdit {
    pub size: bool,
    pub random_shapes: bool,
    pub random_biomes: bool,
    pub perlin_scale_stage_1: bool,
    pub perlin_scale_stage_2: bool,
    pub perlin_scale_stage_3: bool,
    pub mountain_edges: bool,
    pub random_vegetation: bool,
    pub random_resources: bool,
    pub center_lake: bool,
    pub round_edges: bool,
    pub square_edges: bool,
    pub ring_effect: bool,
    pub center_land: bool,
    pub low_ground: bool,
    pub high_ground: bool,
    pub remove_mountains: bool,
    pub cubicles: bool,
}

impl AllowEdit {
    pub const NORMAL: Self = Self {
        size: true,
        random_shapes: true,
        random_biomes: true,
        perlin_scale_stage_1: true,
        perlin_scale_stage_2: true,
        perlin_scale_stage_3: true,
        mountain_edges: true,
        random_vegetation: true,
        random_resources: true,
        center_lake: true,
        round_edges: true,
        square_edges: true,
        ring_effect: true,
        center_land: true,
        low_ground: true,
        high_ground: true,
        remove_mountains: true,
        cubicles: false,
    };

    pub const DISABLED: Self = Self {
        size: false,
        random_shapes: false,
        random_biomes: false,
        perlin_scale_stage_1: false,
        perlin_scale_stage_2: false,
        perlin_scale_stage_3: false,
        mountain_edges: false,
        random_vegetation: false,
        random_resources: false,
        center_lake: false,
        round_edges: false,
        square_edges: false,
        ring_effect: false,
        center_land: false,
        low_ground: false,
        high_ground: false,
        remove_mountains: false,
        cubicles: false,
    };

    const NATURE_ONLY: Self = Self {
        random_biomes: true,
        random_vegetation: true,
        random_resources: true,
        ..Self::DISABLED
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapGenTemplate {
    pub id: TemplateId,
    pub values: MapGenValues,
    pub force_height_to: i32,
    pub freeze_mountains: bool,
    pub special_anthill: bool,
    pub special_checkerboard: bool,
    pub special_cubicles: bool,
    pub perlin_replace: &'static [PerlinReplaceContainer],
    pub allow_edit: AllowEdit,
    pub show_reset_button: bool,
}

impl MapGenTemplate {
    const BASE: Self = Self {
        id: TemplateId::Continent,
        values: MapGenValues::BLANK,
        force_height_to: 0,
        freeze_mountains: false,
        special_anthill: false,
        special_checkerboard: false,
        special_cubicles: false,
        perlin_replace: &[],
        allow_edit: AllowEdit::NORMAL,
        show_reset_button: true,
    };

    pub fn default_values(&self) -> MapGenValues {
        self.values
    }
}

const fn option(replace_height_value: i32, from: Ground, to: Ground) -> PerlinReplaceOption {
    PerlinReplaceOption {
        replace_height_value,
        from,
        to,
    }
}

const SOFTEN_HIGH_SOIL: &[PerlinReplaceContainer] = &[PerlinReplaceContainer {
    scale: 15,
    options: &[option(170, Ground::SoilHigh, Ground::SoilLow)],
}];

const BORING_PLAINS_REPLACE: &[PerlinReplaceContainer] = &[PerlinReplaceContainer {
    scale: 15,
    options: &[
        option(170, Ground::SoilHigh, Ground::SoilLow),
        option(0, Ground::ShallowWaters, Ground::SoilLow),
        option(0, Ground::CloseOcean, Ground::SoilLow),
        option(0, Ground::DeepOcean, Ground::SoilHigh),
    ],
}];

const CHEESE_REPLACE: &[PerlinReplaceContainer] = &[PerlinReplaceContainer {
    scale: 15,
    options: &[
        option(170, Ground::SoilHigh, Ground::ShallowWaters),
        option(170, Ground::Sand, Ground::ShallowWaters),
        option(150, Ground::SoilHigh, Ground::Sand),
        option(130, Ground::SoilLow, Ground::Sand),
        option(80, Ground::SoilHigh, Ground::SoilLow),
    ],
}];

const PERLIN_ALL_STAGES: MapGenValues = MapGenValues {
    main_perlin_noise_stage: true,
    perlin_noise_stage_2: true,
    perlin_noise_stage_3: true,
    ..MapGenValues::BLANK
};

pub const TEMPLATES: [MapGenTemplate; 16] = [
    MapGenTemplate {
        id: TemplateId::Continent,
        freeze_mountains: true,
        values: MapGenValues {
            gradient_round_edges: true,
            add_center_gradient_land: true,
            ring_effect: true,
            random_shapes_amount: 5,
            ..PERLIN_ALL_STAGES
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::BoxWorld,
        freeze_mountains: true,
        values: MapGenValues {
            add_mountain_edges: true,
            add_center_gradient_land: true,
            ring_effect: true,
            random_shapes_amount: 5,
            ..PERLIN_ALL_STAGES
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Islands,
        freeze_mountains: true,
        values: MapGenValues {
            gradient_round_edges: true,
            ring_effect: true,
            random_shapes_amount: 5,
            ..PERLIN_ALL_STAGES
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Toast,
        values: MapGenValues {
            square_edges: true,
            add_center_gradient_land: true,
            remove_mountains: true,
            random_shapes_amount: 5,
            ..PERLIN_ALL_STAGES
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Pancake,
        values: MapGenValues {
            gradient_round_edges: true,
            add_center_gradient_land: true,
            remove_mountains: true,
            ..PERLIN_ALL_STAGES
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::BoringPlains,
        values: MapGenValues {
            add_center_gradient_land: true,
            add_mountain_edges: true,
            remove_mountains: true,
            ..PERLIN_ALL_STAGES
        },
        allow_edit: AllowEdit {
            low_ground: false,
            high_ground: false,
            ..AllowEdit::NORMAL
        },
        perlin_replace: BORING_PLAINS_REPLACE,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Checkerboard,
        special_checkerboard: true,
        values: MapGenValues {
            random_biomes: true,
            add_mountain_edges: true,
            remove_mountains: true,
            ..MapGenValues::BLANK
        },
        allow_edit: AllowEdit::NATURE_ONLY,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Cubicles,
        special_cubicles: true,
        values: MapGenValues {
            random_biomes: true,
            add_mountain_edges: true,
            remove_mountains: true,
            cubicle_size: 2,
            ..MapGenValues::BLANK
        },
        allow_edit: AllowEdit {
            cubicles: true,
            ..AllowEdit::NATURE_ONLY
        },
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::DormantVolcano,
        force_height_to: 125,
        values: MapGenValues {
            perlin_noise_stage_2: true,
            random_biomes: false,
            gradient_round_edges: true,
            add_center_gradient_land: true,
            ..MapGenValues::BLANK
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Cheese,
        force_height_to: 120,
        values: MapGenValues {
            square_edges: true,
            perlin_noise_stage_2: true,
            remove_mountains: true,
            add_center_gradient_land: true,
            ..MapGenValues::BLANK
        },
        perlin_replace: CHEESE_REPLACE,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::BadApple,
        values: MapGenValues {
            gradient_round_edges: true,
            add_center_gradient_land: true,
            center_gradient_mountains: true,
            main_perlin_noise_stage: true,
            ..MapGenValues::BLANK
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Donut,
        force_height_to: 125,
        values: MapGenValues {
            gradient_round_edges: true,
            add_center_gradient_land: true,
            add_center_lake: true,
            perlin_noise_stage_2: true,
            ..MapGenValues::BLANK
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Lasagna,
        values: MapGenValues {
            square_edges: true,
            add_center_gradient_land: true,
            main_perlin_noise_stage: true,
            random_shapes_amount: 5,
            low_ground: true,
            ..MapGenValues::BLANK
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::ChaosPearl,
        values: MapGenValues {
            gradient_round_edges: true,
            add_center_gradient_land: true,
            main_perlin_noise_stage: true,
            low_ground: true,
            ..MapGenValues::BLANK
        },
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Anthill,
        special_anthill: true,
        allow_edit: AllowEdit::NATURE_ONLY,
        perlin_replace: SOFTEN_HIGH_SOIL,
        ..MapGenTemplate::BASE
    },
    MapGenTemplate {
        id: TemplateId::Empty,
        show_reset_button: false,
        allow_edit: AllowEdit::DISABLED,
        ..MapGenTemplate::BASE
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_sits_at_its_own_index() {
        for (index, id) in TEMPLATE_IDS.into_iter().enumerate() {
            assert_eq!(TEMPLATES[index].id, id);
            assert_eq!(id.template().id, id);
            assert_eq!(TemplateId::from_id(id.id()), Some(id));
        }
        assert_eq!(TemplateId::from_id("atlantis"), None);
    }

    #[test]
    fn the_continent_is_the_first_launch_template() {
        let continent = TemplateId::default().template();
        let values = continent.values;

        assert_eq!(continent.id.id(), "continent");
        assert!(continent.freeze_mountains);
        assert!(values.main_perlin_noise_stage);
        assert!(values.perlin_noise_stage_2);
        assert!(values.perlin_noise_stage_3);
        assert!(values.gradient_round_edges);
        assert!(values.add_center_gradient_land);
        assert!(values.ring_effect);
        assert!(values.random_biomes && values.add_vegetation && values.add_resources);
        assert!(!values.square_edges && !values.add_mountain_edges);
        assert_eq!(values.random_shapes_amount, 5);
        assert_eq!(
            (
                values.perlin_scale_stage_1,
                values.perlin_scale_stage_2,
                values.perlin_scale_stage_3
            ),
            (5, 5, 5)
        );
        assert_eq!(continent.perlin_replace.len(), 1);
        assert_eq!(continent.perlin_replace[0].scale, 15);
        assert_eq!(
            continent.perlin_replace[0].options,
            [option(170, Ground::SoilHigh, Ground::SoilLow)]
        );
    }

    #[test]
    fn the_locked_templates_only_open_the_nature_settings() {
        for id in [
            TemplateId::Checkerboard,
            TemplateId::Cubicles,
            TemplateId::Anthill,
        ] {
            let allow = id.template().allow_edit;

            assert!(allow.random_biomes && allow.random_vegetation && allow.random_resources);
            assert!(!allow.size && !allow.random_shapes && !allow.perlin_scale_stage_1);
        }
        assert!(TemplateId::Cubicles.template().allow_edit.cubicles);
        assert!(!TemplateId::Continent.template().allow_edit.cubicles);
        assert_eq!(TemplateId::Empty.template().allow_edit, AllowEdit::DISABLED);
        assert!(!TemplateId::Empty.template().show_reset_button);
        assert!(!TemplateId::BoringPlains.template().allow_edit.low_ground);
    }

    #[test]
    fn the_special_templates_carry_their_own_heights_and_shapes() {
        assert_eq!(TemplateId::DormantVolcano.template().force_height_to, 125);
        assert_eq!(TemplateId::Cheese.template().force_height_to, 120);
        assert_eq!(TemplateId::Donut.template().force_height_to, 125);
        assert!(!TemplateId::DormantVolcano.template().values.random_biomes);
        assert_eq!(TemplateId::Cubicles.template().values.cubicle_size, 2);
        assert_eq!(
            TemplateId::Cheese.template().perlin_replace[0]
                .options
                .len(),
            5
        );
        assert_eq!(
            TemplateId::BoringPlains.template().perlin_replace[0].options[3],
            option(0, Ground::DeepOcean, Ground::SoilHigh)
        );
        assert!(
            TemplateId::Checkerboard
                .template()
                .perlin_replace
                .is_empty()
        );
    }
}
