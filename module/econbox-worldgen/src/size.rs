pub const ZONE_TILES: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum MapSize {
    Tiny,
    Small,
    #[default]
    Standard,
    Large,
    Huge,
    Gigantic,
    Titanic,
    Iceberg,
}

pub const MAP_SIZES: [MapSize; 8] = [
    MapSize::Tiny,
    MapSize::Small,
    MapSize::Standard,
    MapSize::Large,
    MapSize::Huge,
    MapSize::Gigantic,
    MapSize::Titanic,
    MapSize::Iceberg,
];

impl MapSize {
    pub fn id(self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Small => "small",
            Self::Standard => "standard",
            Self::Large => "large",
            Self::Huge => "huge",
            Self::Gigantic => "gigantic",
            Self::Titanic => "titanic",
            Self::Iceberg => "iceberg",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        MAP_SIZES.into_iter().find(|size| size.id() == id)
    }

    pub fn from_zones(zones: u32) -> Option<Self> {
        MAP_SIZES.into_iter().find(|size| size.zones() == zones)
    }

    pub fn zones(self) -> u32 {
        self as u32 + 2
    }

    pub fn tiles(self) -> u32 {
        self.zones() * ZONE_TILES
    }

    pub fn show_warning(self) -> bool {
        self >= Self::Large
    }

    pub fn path_icon(self) -> &'static str {
        match self {
            Self::Tiny => "actor_traits/iconTiny",
            Self::Small => "iconAntBlack",
            Self::Standard => "iconTileSand",
            Self::Large => "iconTileSoil",
            Self::Huge => "iconTileHighSoil",
            Self::Gigantic => "iconTileMountains",
            Self::Titanic => "iconTitanic",
            Self::Iceberg => "iconIceberg",
        }
    }

    pub fn locale_id(self) -> String {
        format!("map_size_{}", self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_size_counts_zones_of_64_tiles() {
        let tiles: Vec<u32> = MAP_SIZES.map(MapSize::tiles).to_vec();

        assert_eq!(tiles, [128, 192, 256, 320, 384, 448, 512, 576]);
        assert_eq!(MapSize::default(), MapSize::Standard);
        assert_eq!(MapSize::Standard.zones(), 4);
    }

    #[test]
    fn the_big_sizes_warn_before_they_are_picked() {
        let warned: Vec<&str> = MAP_SIZES
            .into_iter()
            .filter(|size| size.show_warning())
            .map(MapSize::id)
            .collect();

        assert_eq!(warned, ["large", "huge", "gigantic", "titanic", "iceberg"]);
    }

    #[test]
    fn a_size_is_found_by_its_id_and_its_zones() {
        for size in MAP_SIZES {
            assert_eq!(MapSize::from_id(size.id()), Some(size));
            assert_eq!(MapSize::from_zones(size.zones()), Some(size));
        }
        assert_eq!(MapSize::from_zones(1), None);
        assert_eq!(MapSize::Iceberg.locale_id(), "map_size_iceberg");
    }
}
