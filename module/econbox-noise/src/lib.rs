//! Seeded randomness and classic Perlin noise, matching the numbers WorldBox draws.

mod perlin;
mod randy;

pub use perlin::perlin;
pub use randy::{NetRandom, Randy, XorShift};
