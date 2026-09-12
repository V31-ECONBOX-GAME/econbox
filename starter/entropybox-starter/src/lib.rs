pub use bevy;
pub use entropybox;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use entropybox::VERSION;
    pub use entropybox::config::{Config, ConfigError};
}
