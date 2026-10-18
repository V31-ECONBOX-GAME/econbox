//! Turns the drawn pixels into textures once, so every piece of the bar can share them.

use crate::catalog::{Look, Plate, TabId, buttons, tab};
use crate::tabs::TABS;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};
use bevy::ui::widget::NodeImageMode;
use econbox_art::ui::{self, Picture, TabMark, Well};
use econbox_art::{Glyph, Icon, Ink, SPRITE, Skin, glyph, icon, inked, sprite};
use econbox_brush::{PRESETS, Preset, Shape, mask};
use econbox_clock::SPEEDS;
use std::collections::HashMap;

#[derive(Resource, Debug, Clone)]
pub struct Kit {
    pub bar: Handle<Image>,
    pub tab: Handle<Image>,
    pub tab_on: Handle<Image>,
    pub button: Handle<Image>,
    pub studded: Handle<Image>,
    pub present: Handle<Image>,
    pub selection: Handle<Image>,
    pub strong: Handle<Image>,
    pub thin: Handle<Image>,
    pub arrow: Handle<Image>,
    pub tip_body: Handle<Image>,
    pub tip_head: Handle<Image>,
    pub window: Handle<Image>,
    pub inset: Handle<Image>,
    pub close_tab: Handle<Image>,
    pub back_plate: Handle<Image>,
    pub red: Handle<Image>,
    pub red_well: Handle<Image>,
    pub gilded: Handle<Image>,
    pub icons: HashMap<&'static str, Handle<Image>>,
    pub marks: HashMap<TabId, Handle<Image>>,
    pub brushes: Vec<(Handle<Image>, Vec2)>,
    pub badges: Vec<Handle<Image>>,
    pub close: Handle<Image>,
    pub back: Handle<Image>,
    pub hourglass: Handle<Image>,
    pub pause: Handle<Image>,
    pub play: Handle<Image>,
    pub eye: Handle<Image>,
}

impl Kit {
    pub fn plate(&self, plate: Plate) -> &Handle<Image> {
        match plate {
            Plate::Plain => &self.button,
            Plate::Studded => &self.studded,
            Plate::Red => &self.red,
            Plate::RedWell => &self.red_well,
            Plate::Gilded => &self.gilded,
        }
    }
}

pub fn crisp(picture: &Picture) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: picture.width,
            height: picture.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        picture.bytes(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    image
}

fn square(pixels: Vec<[u8; 4]>) -> Picture {
    Picture {
        width: SPRITE as u32,
        height: SPRITE as u32,
        pixels,
    }
}

pub fn look_pixels(look: Look) -> Vec<[u8; 4]> {
    match look {
        Look::Icon(kind, tint) => match tint.color() {
            Some(base) => inked(
                kind.template(),
                Ink {
                    base,
                    accent: kind.ink().accent,
                },
            ),
            None => icon(kind, None),
        },
        Look::Glyph(kind, base, accent) => {
            let ink = kind.ink();
            glyph(
                kind,
                Some(Ink {
                    base: base.unwrap_or(ink.base),
                    accent: accent.unwrap_or(ink.accent),
                }),
            )
        }
        Look::Creature(species) => sprite(species.stance(), species.mark(), species.skin()),
        Look::Beast(stance, mark, coat, accent) => sprite(stance, mark, Skin { coat, accent }),
    }
}

pub fn sliced(border: f32) -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(border),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

pub fn bar_slices() -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(ui::BAR_EDGE as f32, 10.0),
            max_inset: Vec2::new(ui::BAR_EDGE as f32, 10.0),
        },
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

const SPECIAL_SIZES: [f32; 5] = [14.0, 17.93, 21.4, 23.55, 28.28];

pub fn brush_size(preset: &Preset, side: u32) -> Vec2 {
    if preset.flood {
        return Vec2::splat(24.0);
    }

    let special = PRESETS
        .iter()
        .filter(|other| other.shape == Shape::Scatter)
        .position(|other| other.id == preset.id);

    if let Some(at) = special {
        return Vec2::splat(SPECIAL_SIZES[at]);
    }

    let base = if side < 28 { side as f32 } else { 28.0 };
    let scale = match preset.id {
        "circ_10" | "circ_15" | "diamond_7" => 0.9,
        "sqr_10" | "sqr_15" | "diamond_2" => 0.8,
        "diamond_4" | "diamond_5" => 0.7,
        _ => 1.0,
    };

    Vec2::splat(base * scale)
}

fn badge(label: &str) -> Picture {
    let mut canvas = Picture::blank(28, 28);
    ui::stamp_text(&mut canvas, label, 27, 27);
    canvas
}

pub fn forge(mut commands: Commands, images: Option<ResMut<Assets<Image>>>) {
    let Some(mut images) = images else { return };
    let mut bake = |picture: &Picture| images.add(crisp(picture));

    let mut icons = HashMap::new();
    for tab in TABS {
        for def in buttons(tab.id) {
            icons
                .entry(def.id)
                .or_insert_with(|| bake(&square(look_pixels(def.look))));
        }
    }

    let mut marks = HashMap::new();
    for id in crate::catalog::STRIP {
        let mark = tab(id).mark.unwrap_or(TabMark::Misc);
        marks.insert(id, bake(&ui::tab_mark(mark)));
    }

    let brushes = PRESETS
        .iter()
        .map(|preset| {
            if preset.flood {
                let picture = square(icon(Icon::Bucket, None));
                return (bake(&picture), brush_size(preset, SPRITE as u32));
            }

            let (side, cells) = mask(preset.shape, preset.radius);
            (
                bake(&ui::mask_picture(&cells, side)),
                brush_size(preset, side),
            )
        })
        .collect();

    let badges = SPEEDS
        .iter()
        .map(|speed| bake(&badge(speed.label)))
        .collect();

    commands.insert_resource(Kit {
        bar: bake(&ui::bar()),
        tab: bake(&ui::tab(false)),
        tab_on: bake(&ui::tab(true)),
        button: bake(&ui::button(Well::Plain)),
        studded: bake(&ui::button(Well::Studded)),
        present: bake(&ui::button(Well::Present)),
        selection: bake(&ui::selection()),
        strong: bake(&ui::groove(true)),
        thin: bake(&ui::groove(false)),
        arrow: bake(&ui::arrow()),
        tip_body: bake(&ui::tooltip_body()),
        tip_head: bake(&ui::tooltip_head()),
        window: bake(&ui::window_frame()),
        inset: bake(&ui::inset_panel()),
        close_tab: bake(&ui::close_tab()),
        back_plate: bake(&ui::red_plate(32, 68)),
        red: bake(&ui::red_plate(32, 32)),
        red_well: bake(&ui::red_well(32, 32)),
        gilded: bake(&ui::gilded(32, 32)),
        icons,
        marks,
        brushes,
        badges,
        close: bake(&square(glyph(Glyph::Close, None))),
        back: bake(&square(glyph(Glyph::BackArrow, None))),
        hourglass: bake(&square(icon(Icon::Hourglass, None))),
        pause: bake(&square(icon(Icon::Pause, None))),
        play: bake(&square(icon(Icon::Play, None))),
        eye: bake(&square(glyph(Glyph::Eye, None))),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brush_icons_shrink_the_way_the_reference_window_shows_them() {
        let circ = |id: &str| PRESETS.iter().find(|preset| preset.id == id).unwrap();

        assert_eq!(brush_size(circ("circ_0"), 1), Vec2::splat(1.0));
        assert_eq!(brush_size(circ("circ_5"), 11), Vec2::splat(11.0));
        assert_eq!(brush_size(circ("circ_15"), 31), Vec2::splat(28.0 * 0.9));
        assert_eq!(brush_size(circ("diamond_4"), 23), Vec2::splat(23.0 * 0.7));
        assert_eq!(brush_size(circ("scatter_5"), 69), Vec2::splat(28.28));
    }

    #[test]
    fn every_look_paints_a_full_sprite() {
        for tab in TABS {
            for def in buttons(tab.id) {
                let pixels = look_pixels(def.look);
                assert_eq!(pixels.len(), SPRITE * SPRITE, "{}", def.id);
                assert!(pixels.iter().any(|px| px[3] > 0), "{}", def.id);
            }
        }
    }
}
