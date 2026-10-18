//! Where every item of a tab sits, in canvas units measured from the tab origin.

use crate::catalog::{Element, Item};

pub const CANVAS_WIDTH: f32 = 285.0;
pub const CANVAS_HEIGHT: f32 = 500.0;

pub const BUTTON: f32 = 32.0;
pub const ICON: f32 = 24.0;
pub const PITCH: f32 = 36.0;
pub const START: f32 = 20.0;
pub const GAP: f32 = 4.0;
pub const HALF: f32 = 9.6;
pub const END_PAD: f32 = 5.0;
pub const TOP_ROW: f32 = 32.0;
pub const BOTTOM_ROW: f32 = -4.0;
pub const LINE_TOP: f32 = 37.2;
pub const LINE_HEIGHT: f32 = 3.0 * 48.3;
pub const STRONG_LINE: f32 = 5.0;
pub const THIN_LINE: f32 = 2.0;
pub const BACK_HEIGHT: f32 = 68.0;
pub const ELEMENT_WIDTH: f32 = 128.0;
pub const AGE_HEIGHT: f32 = 68.2132;
pub const INFO_HEIGHT: f32 = 68.0;

pub const BAR_BOTTOM: f32 = -33.4;
pub const BAR_HEIGHT: f32 = 120.0;
pub const BAR_TOP: f32 = BAR_BOTTOM + BAR_HEIGHT;
pub const TAB_ORIGIN: f32 = 42.4;
pub const STRIP_BOTTOM: f32 = 80.6;
pub const STRIP_CELL_WIDTH: f32 = 43.0;
pub const STRIP_CELL_HEIGHT: f32 = 18.0;
pub const SCROLL_TOP: f32 = 89.8;
pub const CONTENT_LEFT: f32 = -1.9;
pub const ARROW_WIDTH: f32 = 10.0;
pub const ARROW_HIDE: f32 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub index: usize,
    pub x: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Place {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.top - self.height
    }

    pub fn centre(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.top - self.height * 0.5)
    }
}

pub fn line_width(item: &Item) -> Option<f32> {
    match item {
        Item::Line | Item::End => Some(STRONG_LINE),
        Item::Thin => Some(THIN_LINE),
        _ => None,
    }
}

pub fn element_size(element: Element) -> (f32, f32) {
    match element {
        Element::AgeClock => (ELEMENT_WIDTH, AGE_HEIGHT),
        Element::PanelInfo => (ELEMENT_WIDTH, INFO_HEIGHT),
    }
}

pub fn arrange(items: &[Item]) -> (Vec<Place>, f32) {
    let mut places = Vec::new();
    let mut x = START;
    let mut filled = 0usize;
    let mut widest: f32 = 0.0;

    for (index, item) in items.iter().enumerate() {
        match item {
            Item::Half => {
                if filled > 0 {
                    x += PITCH;
                    filled = 0;
                }
                x += HALF;
            }
            Item::Line | Item::Thin | Item::End | Item::Element(_) => {
                if filled > 0 {
                    x += PITCH;
                    filled = 0;
                }

                let (top, width, height) = match item {
                    Item::Element(element) => {
                        let (width, height) = element_size(*element);
                        (TOP_ROW, width, height)
                    }
                    _ => (
                        LINE_TOP,
                        line_width(item).unwrap_or(STRONG_LINE),
                        LINE_HEIGHT,
                    ),
                };

                let place = Place {
                    index,
                    x,
                    top,
                    width,
                    height,
                };
                widest = widest.max(place.right());
                places.push(place);
                x += width + GAP;
            }
            Item::Space | Item::Button(_) | Item::Back => {
                let opening = filled.is_multiple_of(2);
                let advance = opening && filled > 0;
                let lower = !opening;
                filled += 1;

                if advance {
                    x += PITCH;
                }

                if matches!(item, Item::Space) {
                    continue;
                }

                let (top, height) = match item {
                    Item::Back => (TOP_ROW, BACK_HEIGHT),
                    _ if lower => (BOTTOM_ROW, BUTTON),
                    _ => (TOP_ROW, BUTTON),
                };

                let place = Place {
                    index,
                    x,
                    top,
                    width: BUTTON,
                    height,
                };
                widest = widest.max(place.right());
                places.push(place);
            }
        }
    }

    (places, widest + END_PAD)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{TabId, tab};

    fn place_of(id: TabId, wanted: impl Fn(&Item) -> bool) -> Vec<Place> {
        let items = tab(id).items;
        let (places, _) = arrange(items);
        places
            .into_iter()
            .filter(|place| wanted(&items[place.index]))
            .collect()
    }

    fn named(id: TabId, button: &str) -> Place {
        place_of(
            id,
            |item| matches!(item, Item::Button(def) if def.id == button),
        )[0]
    }

    fn is_line(item: &Item) -> bool {
        matches!(item, Item::Line | Item::Thin | Item::End)
    }

    #[test]
    fn the_main_tab_matches_the_reference_layout() {
        assert_eq!(named(TabId::Main, "pause").centre(), (36.0, 16.0));
        assert_eq!(named(TabId::Main, "clock").centre(), (36.0, -20.0));

        let lines: Vec<f32> = place_of(TabId::Main, is_line)
            .iter()
            .map(|place| place.x)
            .collect();
        assert_eq!(lines[0], 56.0);
        assert_eq!(lines[1], 173.0);
        assert_eq!(lines[2], 215.0);
        assert_eq!(lines[3], 329.0);
        assert!((lines[4] - 611.6).abs() < 1e-3, "{}", lines[4]);

        let elements = place_of(TabId::Main, |item| matches!(item, Item::Element(_)));
        assert_eq!(elements[0].x, 338.0);
        assert!((elements[1].x - 479.6).abs() < 1e-3);
    }

    #[test]
    fn category_tabs_open_with_a_tall_back_button_then_the_time_controls() {
        let back = place_of(TabId::Creation, |item| matches!(item, Item::Back))[0];

        assert_eq!(
            (back.x, back.top, back.height),
            (20.0, TOP_ROW, BACK_HEIGHT)
        );
        assert_eq!(named(TabId::Creation, "pause").x, 56.0);
        assert_eq!(named(TabId::Creation, "clock").top, BOTTOM_ROW);
    }

    #[test]
    fn buttons_fill_columns_top_first_then_bottom() {
        let deep = named(TabId::Creation, "tile_deep_ocean");
        let sand = named(TabId::Creation, "tile_sand");
        let shallow = named(TabId::Creation, "tile_shallow_waters");

        assert_eq!(shallow.x, deep.x + PITCH);
        assert_eq!(sand.x, shallow.x);
        assert_eq!(sand.top, BOTTOM_ROW);
        assert_eq!(shallow.top, TOP_ROW);
        assert_eq!(sand.centre(), (153.0, -20.0));
    }

    #[test]
    fn the_content_is_as_wide_as_its_last_item_plus_padding() {
        let (places, width) = arrange(tab(TabId::Main).items);
        let last = places.iter().map(Place::right).fold(0.0, f32::max);

        assert_eq!(width, last + END_PAD);
    }
}
