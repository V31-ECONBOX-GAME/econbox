use crate::view::{nearest, world_size};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use econbox_tilemap::TileMap;

pub const BORDER_LAYER: f32 = 0.25;
pub const BORDER_MAX: f32 = 0.42;
pub const BORDER_STEP: f32 = 0.02;
pub const BORDER_ALPHA_INTERVAL: f32 = 0.02;
pub const BORDER_MARCH_INTERVAL: f32 = 0.12;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PowerSelected(pub bool);

#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct MapBorder {
    pub alpha: f32,
    pub state: u8,
    pub alpha_timer: f32,
    pub march_timer: f32,
    pub width: u32,
    pub height: u32,
}

pub fn perimeter(width: u32, height: u32) -> Vec<(u32, u32)> {
    let mut order: Vec<(u32, u32)> = Vec::new();
    let add = |cells: Vec<(u32, u32)>, order: &mut Vec<(u32, u32)>| {
        let fresh: Vec<(u32, u32)> = cells.into_iter().filter(|c| !order.contains(c)).collect();
        order.extend(fresh);
    };
    add((0..width).map(|x| (x, 0)).collect(), &mut order);
    add((0..height).map(|y| (width - 1, y)).collect(), &mut order);
    add((0..width).map(|x| (x, height - 1)).collect(), &mut order);
    let mut left: Vec<(u32, u32)> = (0..height)
        .map(|y| (0, y))
        .filter(|c| !order.contains(c))
        .collect();
    left.reverse();
    order.extend(left);
    order
}

pub fn dashes(width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0_u8; (width * height * 4) as usize];
    let mut run = 0;
    for (x, y_up) in perimeter(width, height) {
        if run < 3 {
            let row = height - 1 - y_up;
            let at = ((row * width + x) * 4) as usize;
            data[at..at + 4].copy_from_slice(&[255, 255, 255, 255]);
            run += 1;
        } else {
            run = 0;
        }
    }
    data
}

pub fn border_image(map: &TileMap) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: map.width.max(1),
            height: map.height.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        dashes(map.width.max(1), map.height.max(1)),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    nearest(&mut image);
    image
}

pub fn white_over_alpha(alpha: f32) -> f32 {
    let linear = |c: f32| Color::srgb(c, c, c).to_linear().red;
    let grey = 0.5;
    let target = linear(grey + (1.0 - grey) * alpha);
    (target - linear(grey)) / (1.0 - linear(grey))
}

impl MapBorder {
    pub fn update(&mut self, seconds: f32, selected: bool) {
        self.alpha_timer -= seconds;
        while self.alpha_timer <= 0.0 {
            self.alpha_timer += BORDER_ALPHA_INTERVAL;
            self.alpha = if selected {
                (self.alpha + BORDER_STEP).min(BORDER_MAX)
            } else {
                (self.alpha - BORDER_STEP).max(0.0)
            };
        }

        self.march_timer -= seconds;
        while self.march_timer <= 0.0 {
            self.march_timer += BORDER_MARCH_INTERVAL;
            if self.alpha != 0.0 {
                self.state = (self.state + 1) % 4;
            }
        }
    }

    pub fn flips(&self) -> (bool, bool) {
        match self.state {
            0 => (false, false),
            1 => (true, false),
            2 => (true, true),
            _ => (false, true),
        }
    }
}

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        MapBorder::default(),
        Sprite::default(),
        Transform::from_xyz(0.0, 0.0, BORDER_LAYER),
        Visibility::Hidden,
    ));
}

pub fn animate(
    time: Res<Time>,
    selected: Res<PowerSelected>,
    mut images: ResMut<Assets<Image>>,
    map: Option<Res<TileMap>>,
    mut borders: Query<(&mut MapBorder, &mut Sprite, &mut Visibility)>,
) {
    let Some(map) = map else { return };

    for (mut border, mut sprite, mut visibility) in &mut borders {
        if border.width != map.width || border.height != map.height {
            sprite.image = images.add(border_image(&map));
            sprite.custom_size = Some(world_size(&map));
            border.width = map.width;
            border.height = map.height;
        }

        border.update(time.delta_secs(), selected.0);
        let (flip_x, flip_y) = border.flips();
        sprite.flip_x = flip_x;
        sprite.flip_y = flip_y;
        sprite.color = Color::srgba(1.0, 1.0, 1.0, white_over_alpha(border.alpha));
        visibility.set_if_neq(if border.alpha > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_border_runs_round_the_map_in_dashes_of_three() {
        let data = dashes(10, 6);
        let lit = |x: u32, row: u32| data[((row * 10 + x) * 4 + 3) as usize] == 255;

        let bottom: Vec<bool> = (0..10).map(|x| lit(x, 5)).collect();
        assert_eq!(
            bottom,
            [true, true, true, false, true, true, true, false, true, true]
        );
        assert!((1..9).all(|x| (1..5).all(|row| !lit(x, row))));
        assert_eq!(perimeter(10, 6).len(), 2 * 10 + 2 * 4);
    }

    #[test]
    fn selecting_a_power_fades_the_border_in_to_its_cap_and_marches_it() {
        let mut border = MapBorder::default();
        for _ in 0..100 {
            border.update(0.02, true);
        }
        assert!((border.alpha - BORDER_MAX).abs() < 1e-5);

        let before = border.state;
        border.update(BORDER_MARCH_INTERVAL, true);
        assert_ne!(border.state, before);

        for _ in 0..100 {
            border.update(0.02, false);
        }
        assert_eq!(border.alpha, 0.0);
    }

    #[test]
    fn a_white_overlay_keeps_its_gamma_weight_in_linear_blending() {
        assert!(white_over_alpha(0.0).abs() < 1e-6);
        assert!((white_over_alpha(1.0) - 1.0).abs() < 1e-5);
        let linear = |c: f32| Color::srgb(c, c, c).to_linear().red;
        let a = white_over_alpha(0.42);
        let blended = linear(0.5) + (1.0 - linear(0.5)) * a;
        assert!((blended - linear(0.5 + 0.5 * 0.42)).abs() < 1e-5);
    }
}
