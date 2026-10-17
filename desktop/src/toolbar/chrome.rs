//! The plating: nine-sliced panels, sockets, and the icon atlas they hold.

use super::{Action, CLEAR, PIP, SLOT_SIZE, STACK, TABS, Tab};
use crate::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{BorderRect, SliceScaleMode, TextureSlicer};
use bevy::ui::widget::NodeImageMode;

#[derive(Resource, Debug, Clone)]
pub(super) struct Chrome {
    pub(super) panel: Handle<Image>,
    pub(super) slot: Handle<Image>,
    pub(super) tab: Handle<Image>,
    pub(super) tab_on: Handle<Image>,
    pub(super) button: Handle<Image>,
    pub(super) rail: Handle<Image>,
    pub(super) cap: Handle<Image>,
}

#[derive(Resource, Debug, Clone)]
pub(super) struct ToolArt {
    pub(super) image: Handle<Image>,
    pub(super) layout: Handle<TextureAtlasLayout>,
}

const ACTION_ICONS: [(Action, Icon); 7] = [
    (Action::Undo, Icon::Undo),
    (Action::Play, Icon::Pause),
    (Action::Speed, Icon::Hourglass),
    (Action::Shape, Icon::Round),
    (Action::Shrink, Icon::Minus),
    (Action::Grow, Icon::Plus),
    (Action::Flood, Icon::Bucket),
];

const EXTRA_ICONS: [Icon; 3] = [Icon::Play, Icon::Square, Icon::Droplet];

fn painted_tools() -> Vec<Tool> {
    TABS.iter()
        .flat_map(|tab| tab.tools())
        .filter(|tool| tool.icon().is_some())
        .collect()
}

fn frames() -> Vec<Vec<[u8; 4]>> {
    painted_tools()
        .iter()
        .filter_map(|tool| tool.icon())
        .map(|(kind, tint)| icon(kind, tint))
        .chain(ACTION_ICONS.iter().map(|(_, kind)| icon(*kind, None)))
        .chain(EXTRA_ICONS.iter().map(|kind| icon(*kind, None)))
        .chain(TABS.iter().map(|tab| icon(tab.badge(), None)))
        .collect()
}

pub(super) fn tool_slot(tool: Tool) -> Option<usize> {
    painted_tools().iter().position(|found| *found == tool)
}

pub(super) fn action_slot(action: Action) -> usize {
    painted_tools().len()
        + ACTION_ICONS
            .iter()
            .position(|(found, _)| *found == action)
            .unwrap_or(0)
}

pub(super) fn extra_slot(kind: Icon) -> usize {
    painted_tools().len()
        + ACTION_ICONS.len()
        + EXTRA_ICONS
            .iter()
            .position(|found| *found == kind)
            .unwrap_or(0)
}

pub(super) fn badge_slot(tab: Tab) -> usize {
    painted_tools().len()
        + ACTION_ICONS.len()
        + EXTRA_ICONS.len()
        + TABS.iter().position(|found| *found == tab).unwrap_or(0)
}

fn crisp(width: u32, height: u32, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });

    image
}

fn sliced(left: f32, top: f32, right: f32, bottom: f32) -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            min_inset: Vec2::new(left, top),
            max_inset: Vec2::new(right, bottom),
        },
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 2.0,
    })
}

pub(super) fn forge(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
) {
    let (Some(mut images), Some(mut layouts)) = (images, layouts) else {
        return;
    };

    let mut forged = |rows: &[&str], metal: art::Metal| {
        let (data, width, height) = chrome(rows, metal);
        images.add(crisp(width, height, data))
    };

    commands.insert_resource(Chrome {
        panel: forged(&art::RAIL_PIECE, art::FRAME),
        slot: forged(&art::SLOT_PIECE, art::SOCKET),
        tab: forged(&art::TAB_PIECE, art::LEAF),
        tab_on: forged(&art::TAB_PIECE, art::EMBER),
        button: forged(&art::DIVIDER_PIECE, art::EMBER),
        rail: forged(&art::DIVIDER_PIECE, art::FRAME),
        cap: forged(&art::CAP_PIECE, art::FRAME),
    });

    let drawn = frames();
    let (data, width, height) = atlas(&drawn);

    commands.insert_resource(ToolArt {
        image: images.add(crisp(width, height, data)),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(SPRITE as u32),
            drawn.len() as u32,
            1,
            None,
            None,
        )),
    });
}

pub(super) fn plated(handle: &Handle<Image>, mode: NodeImageMode) -> ImageNode {
    ImageNode {
        image: handle.clone(),
        image_mode: mode,
        ..default()
    }
}

pub(super) fn face(metal: art::Metal) -> BackgroundColor {
    let [r, g, b] = metal.face;
    BackgroundColor(Color::srgb_u8(r, g, b))
}

pub(super) fn socket_slice() -> NodeImageMode {
    sliced(6.0, 6.0, 6.0, 6.0)
}

pub(super) fn panel_slice() -> NodeImageMode {
    sliced(10.0, 10.0, 10.0, 6.0)
}

pub(super) fn tab_slice() -> NodeImageMode {
    sliced(10.0, 9.0, 10.0, 3.0)
}

pub(super) fn post_slice() -> NodeImageMode {
    sliced(5.0, 8.0, 5.0, 8.0)
}

pub(super) fn cap_slice() -> NodeImageMode {
    sliced(6.0, 9.0, 6.0, 9.0)
}

pub(super) fn socket(chrome: &Chrome, display: Display) -> impl Bundle {
    (
        Button,
        ActivateOnPress,
        PointerHovered::default(),
        Node {
            width: Val::Px(SLOT_SIZE),
            height: Val::Px(SLOT_SIZE),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display,
            ..default()
        },
        plated(&chrome.slot, socket_slice()),
        face(art::SOCKET),
        BorderColor::all(CLEAR),
    )
}

pub(super) fn pip() -> Node {
    Node {
        width: Val::Px(PIP),
        height: Val::Px(PIP),
        ..default()
    }
}

pub(super) fn rail(chrome: &Chrome, display: Display) -> impl Bundle {
    (
        Node {
            width: Val::Px(10.0),
            height: Val::Px(STACK + 6.0),
            margin: UiRect::right(Val::Px(4.0)),
            display,
            ..default()
        },
        plated(&chrome.rail, post_slice()),
        face(art::FRAME),
    )
}
