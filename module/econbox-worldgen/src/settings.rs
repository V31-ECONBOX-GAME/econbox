use crate::template::{MapGenTemplate, MapGenValues};

pub const MANY_STEP: i32 = 5;

#[derive(Clone, Copy)]
pub struct MapGenSetting {
    pub id: &'static str,
    pub is_switch: bool,
    pub min_value: i32,
    pub max_value: i32,
    pub allowed: fn(&MapGenTemplate) -> bool,
    pub get: fn(&MapGenValues) -> i32,
    pub set: fn(&mut MapGenValues, i32),
}

impl std::fmt::Debug for MapGenSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapGenSetting")
            .field("id", &self.id)
            .field("is_switch", &self.is_switch)
            .field("min_value", &self.min_value)
            .field("max_value", &self.max_value)
            .finish()
    }
}

impl MapGenSetting {
    pub fn locale_id(&self) -> &'static str {
        self.id
    }

    pub fn is_allowed(&self, template: &MapGenTemplate) -> bool {
        (self.allowed)(template)
    }

    pub fn value(&self, values: &MapGenValues) -> i32 {
        (self.get)(values)
    }

    pub fn increase(&self, values: &mut MapGenValues, many: bool) {
        let mut value = (self.get)(values) + if many { MANY_STEP } else { 1 };

        if value > self.max_value {
            value = self.min_value;
        }

        (self.set)(values, value);
    }

    pub fn decrease(&self, values: &mut MapGenValues, many: bool) {
        let mut value = (self.get)(values) - if many { MANY_STEP } else { 1 };

        if value < self.min_value {
            value = self.max_value;
        }

        (self.set)(values, value);
    }

    pub fn switch(&self, values: &mut MapGenValues) {
        let value = if (self.get)(values) == 0 { 1 } else { 0 };

        (self.set)(values, value);
    }
}

pub fn setting(id: &str) -> Option<&'static MapGenSetting> {
    SETTINGS.iter().find(|setting| setting.id == id)
}

const fn number(
    id: &'static str,
    min_value: i32,
    max_value: i32,
    allowed: fn(&MapGenTemplate) -> bool,
    get: fn(&MapGenValues) -> i32,
    set: fn(&mut MapGenValues, i32),
) -> MapGenSetting {
    MapGenSetting {
        id,
        is_switch: false,
        min_value,
        max_value,
        allowed,
        get,
        set,
    }
}

const fn switch(
    id: &'static str,
    allowed: fn(&MapGenTemplate) -> bool,
    get: fn(&MapGenValues) -> i32,
    set: fn(&mut MapGenValues, i32),
) -> MapGenSetting {
    MapGenSetting {
        id,
        is_switch: true,
        min_value: 0,
        max_value: 0,
        allowed,
        get,
        set,
    }
}

pub const SETTINGS: [MapGenSetting; 18] = [
    number(
        "gen_perlin_scale_stage_1",
        0,
        30,
        |t| t.allow_edit.perlin_scale_stage_1,
        |v| v.perlin_scale_stage_1,
        |v, n| v.perlin_scale_stage_1 = n,
    ),
    number(
        "gen_perlin_scale_stage_2",
        0,
        30,
        |t| t.allow_edit.perlin_scale_stage_1,
        |v| v.perlin_scale_stage_2,
        |v, n| v.perlin_scale_stage_2 = n,
    ),
    number(
        "gen_perlin_scale_stage_3",
        0,
        30,
        |t| t.allow_edit.perlin_scale_stage_1,
        |v| v.perlin_scale_stage_3,
        |v, n| v.perlin_scale_stage_3 = n,
    ),
    number(
        "gen_random_shapes",
        0,
        40,
        |t| t.allow_edit.random_shapes,
        |v| v.random_shapes_amount,
        |v, n| v.random_shapes_amount = n,
    ),
    number(
        "gen_cubicles_sizes",
        2,
        15,
        |t| t.allow_edit.cubicles,
        |v| v.cubicle_size,
        |v, n| v.cubicle_size = n,
    ),
    switch(
        "gen_random_biomes",
        |t| t.allow_edit.random_biomes,
        |v| i32::from(v.random_biomes),
        |v, n| v.random_biomes = n == 1,
    ),
    switch(
        "gen_mountain_edges",
        |t| t.allow_edit.mountain_edges,
        |v| i32::from(v.add_mountain_edges),
        |v, n| v.add_mountain_edges = n == 1,
    ),
    switch(
        "gen_add_vegetation",
        |t| t.allow_edit.random_vegetation,
        |v| i32::from(v.add_vegetation),
        |v, n| v.add_vegetation = n == 1,
    ),
    switch(
        "gen_add_resources",
        |t| t.allow_edit.random_resources,
        |v| i32::from(v.add_resources),
        |v, n| v.add_resources = n == 1,
    ),
    switch(
        "gen_add_center_lake",
        |t| t.allow_edit.center_lake,
        |v| i32::from(v.add_center_lake),
        |v, n| v.add_center_lake = n == 1,
    ),
    switch(
        "gen_add_center_land",
        |t| t.allow_edit.center_land,
        |v| i32::from(v.add_center_gradient_land),
        |v, n| v.add_center_gradient_land = n == 1,
    ),
    switch(
        "gen_round_edges",
        |t| t.allow_edit.round_edges,
        |v| i32::from(v.gradient_round_edges),
        |v, n| v.gradient_round_edges = n == 1,
    ),
    switch(
        "gen_square_edges",
        |t| t.allow_edit.square_edges,
        |v| i32::from(v.square_edges),
        |v, n| v.square_edges = n == 1,
    ),
    switch(
        "gen_ring_effect",
        |t| t.allow_edit.ring_effect,
        |v| i32::from(v.ring_effect),
        |v, n| v.ring_effect = n == 1,
    ),
    switch(
        "gen_low_ground",
        |t| t.allow_edit.low_ground,
        |v| i32::from(v.low_ground),
        |v, n| v.low_ground = n == 1,
    ),
    switch(
        "gen_high_ground",
        |t| t.allow_edit.high_ground,
        |v| i32::from(v.high_ground),
        |v, n| v.high_ground = n == 1,
    ),
    switch(
        "gen_remove_mountains",
        |t| t.allow_edit.remove_mountains,
        |v| i32::from(v.remove_mountains),
        |v, n| v.remove_mountains = n == 1,
    ),
    switch(
        "gen_forbidden_knowledge",
        |_| false,
        |v| i32::from(v.forbidden_knowledge_start),
        |v, n| v.forbidden_knowledge_start = n == 1,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TemplateId;

    #[test]
    fn the_settings_keep_the_worldbox_ids_and_ranges() {
        let ids: Vec<&str> = SETTINGS.iter().map(|setting| setting.id).collect();

        assert_eq!(ids.len(), 18);
        assert_eq!(ids[0], "gen_perlin_scale_stage_1");
        assert_eq!(ids[17], "gen_forbidden_knowledge");

        let shapes = setting("gen_random_shapes").expect("random shapes");
        assert_eq!((shapes.min_value, shapes.max_value), (0, 40));
        let cubicles = setting("gen_cubicles_sizes").expect("cubicles");
        assert_eq!((cubicles.min_value, cubicles.max_value), (2, 15));
        assert!(setting("gen_round_edges").expect("round edges").is_switch);
    }

    #[test]
    fn stepping_past_either_end_wraps_around() {
        let scale = setting("gen_perlin_scale_stage_1").expect("scale");
        let mut values = MapGenValues {
            perlin_scale_stage_1: 30,
            ..MapGenValues::default()
        };

        scale.increase(&mut values, false);
        assert_eq!(values.perlin_scale_stage_1, 0);

        scale.decrease(&mut values, false);
        assert_eq!(values.perlin_scale_stage_1, 30);

        scale.decrease(&mut values, true);
        assert_eq!(values.perlin_scale_stage_1, 25);
    }

    #[test]
    fn a_switch_flips_its_flag() {
        let ring = setting("gen_ring_effect").expect("ring");
        let mut values = MapGenValues::default();

        assert_eq!(ring.value(&values), 1);
        ring.switch(&mut values);
        assert!(!values.ring_effect);
        ring.switch(&mut values);
        assert!(values.ring_effect);
    }

    #[test]
    fn the_template_decides_which_settings_can_change() {
        let continent = TemplateId::Continent.template();
        let checkerboard = TemplateId::Checkerboard.template();
        let biomes = setting("gen_random_biomes").expect("biomes");
        let shapes = setting("gen_random_shapes").expect("shapes");
        let stage_3 = setting("gen_perlin_scale_stage_3").expect("stage 3");

        assert!(shapes.is_allowed(continent));
        assert!(!shapes.is_allowed(checkerboard));
        assert!(biomes.is_allowed(checkerboard));
        assert!(stage_3.is_allowed(continent));
        assert!(
            !setting("gen_cubicles_sizes")
                .expect("cubicles")
                .is_allowed(continent)
        );
        assert!(
            !setting("gen_forbidden_knowledge")
                .expect("curse")
                .is_allowed(continent)
        );
    }
}
