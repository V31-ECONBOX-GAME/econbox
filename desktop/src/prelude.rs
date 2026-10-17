//! The vocabulary every module in this crate reaches for.

pub use bevy::prelude::*;

pub use bevy::picking::hover::Hovered as PointerHovered;
pub use bevy::ui::Pressed;
pub use bevy::ui_widgets::{Activate, ActivateOnPress, Button};

pub use econbox::config::{Config, ConfigError};

pub use econbox_art::{self as art, Icon, SPRITE, atlas, chrome, icon};
pub use econbox_brush::{Brush, PointerOverUi, Shape, Tool, Undo, UndoStack};
pub use econbox_camera::MapView;
pub use econbox_clock::WorldClock;
pub use econbox_creature::{Census, CreatureArt, Kind, SPECIES, Species};
pub use econbox_cursor::Hovered;
pub use econbox_tile::{Ground, Overlay, Tile};
pub use econbox_tilemap::{TileMap, TilePos};
pub use econbox_tileview::{Regenerate, WorldSeed};
