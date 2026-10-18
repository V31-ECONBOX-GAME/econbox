//! The two panels on the main tab: the world calendar and the headcount.

use crate::art::{Kit, sliced};
use crate::bar::px;
use crate::catalog::Element;
use bevy::prelude::*;
use econbox_clock::WorldClock;
use econbox_creature::{Census, Kind};
use econbox_tilemap::TileMap;

pub const MONTH: f32 = 6.0;
pub const MONTHS: u32 = 12;

const CALENDAR: Color = Color::srgb(0.0, 1.0, 0.8213);
const NAME: Color = Color::srgb(0.6784, 0.6784, 0.6784);
const VALUE: Color = Color::srgb(1.0, 0.7373, 0.4);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readout {
    Year,
    Month,
    YearBar,
    MonthBar,
    Next,
    Population,
    Beasts,
    Deaths,
    Infected,
    Buildings,
    Vegetation,
}

pub fn calendar(elapsed: f32) -> (u32, u32, f32, f32) {
    let months = (elapsed / MONTH).max(0.0);
    let whole = months.floor() as u32;
    let year = whole / MONTHS + 1;
    let month = whole % MONTHS + 1;
    let through_month = months.fract();
    let through_year = ((whole % MONTHS) as f32 + through_month) / MONTHS as f32;

    (year, month, through_year, through_month)
}

pub fn greenery(map: &TileMap) -> usize {
    map.tiles
        .iter()
        .filter(|tile| tile.overlay.is_living())
        .count()
}

fn label(text: &str, size: f32, color: Color) -> impl Bundle {
    (
        Pickable::IGNORE,
        Text::new(text),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

fn bar_track(kit: &Kit, top: f32, fill: Readout, color: Color) -> impl Bundle {
    (
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: px(10.0),
            right: px(10.0),
            top: px(top),
            height: px(7.0),
            padding: UiRect::all(px(1.5)),
            ..default()
        },
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(3.0),
            ..default()
        },
        children![(
            fill,
            Pickable::IGNORE,
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(color),
        )],
    )
}

pub fn spawn_element(parent: &mut ChildSpawnerCommands, kit: &Kit, element: Element, node: Node) {
    let node = match element {
        Element::PanelInfo => Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            padding: UiRect::axes(px(6.0), px(2.0)),
            ..node
        },
        Element::AgeClock => node,
    };

    let mut panel = parent.spawn((
        node,
        ImageNode {
            image: kit.inset.clone(),
            image_mode: sliced(4.0),
            ..default()
        },
    ));

    match element {
        Element::AgeClock => {
            panel.with_children(|panel| {
                panel.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        right: px(0.0),
                        top: px(3.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![label("World Time", 7.0, CALENDAR)],
                ));
                panel.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(10.0),
                        right: px(10.0),
                        top: px(15.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    children![
                        (Readout::Year, label("Year 1", 6.5, Color::WHITE)),
                        (Readout::Month, label("Month 1", 6.5, Color::WHITE)),
                    ],
                ));
                panel.spawn(bar_track(
                    kit,
                    25.0,
                    Readout::YearBar,
                    Color::srgb(0.1, 0.6, 0.7),
                ));
                panel.spawn((
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        right: px(0.0),
                        top: px(38.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![(Readout::Next, label("", 5.5, NAME))],
                ));
                panel.spawn(bar_track(
                    kit,
                    49.0,
                    Readout::MonthBar,
                    Color::srgb(0.95, 0.75, 0.25),
                ));
            });
        }
        Element::PanelInfo => {
            let rows = [
                ("Population", Readout::Population),
                ("Beasts", Readout::Beasts),
                ("Deaths", Readout::Deaths),
                ("Infected", Readout::Infected),
                ("Buildings", Readout::Buildings),
                ("Vegetation", Readout::Vegetation),
            ];

            panel.with_children(|panel| {
                for (name, readout) in rows {
                    panel.spawn((
                        Pickable::IGNORE,
                        Node {
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        children![label(name, 6.5, NAME), (readout, label("0", 6.5, VALUE)),],
                    ));
                }
            });
        }
    }
}

pub fn refresh(
    clock: Res<WorldClock>,
    census: Res<Census>,
    map: Option<Res<TileMap>>,
    mut green: Local<usize>,
    mut texts: Query<(&Readout, &mut Text)>,
    mut fills: Query<(&Readout, &mut Node), Without<Text>>,
) {
    if let Some(map) = &map
        && map.is_changed()
    {
        *green = greenery(map);
    }

    let (year, month, through_year, through_month) = calendar(clock.elapsed);
    let folk = census.of_kind(Kind::Folk);
    let left = ((1.0 - through_month) * MONTH / clock.speed().multiplier).ceil() as u32;

    for (readout, mut text) in &mut texts {
        let wanted = match readout {
            Readout::Year => format!("Year {year}"),
            Readout::Month => format!("Month {month}"),
            Readout::Next => format!("Next month in {left}s"),
            Readout::Population => folk.to_string(),
            Readout::Beasts => (census.alive - folk).to_string(),
            Readout::Deaths => census.deaths.to_string(),
            Readout::Infected | Readout::Buildings => "-".to_string(),
            Readout::Vegetation => green.to_string(),
            Readout::YearBar | Readout::MonthBar => continue,
        };

        if text.as_str() != wanted {
            **text = wanted;
        }
    }

    for (readout, mut node) in &mut fills {
        let share = match readout {
            Readout::YearBar => through_year,
            Readout::MonthBar => through_month,
            _ => continue,
        };
        let wanted = Val::Percent(share * 100.0);

        if node.width != wanted {
            node.width = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Flora, Ground, Overlay, Tile};

    #[test]
    fn the_calendar_rolls_months_into_years() {
        assert_eq!(calendar(0.0).0, 1);
        assert_eq!(calendar(0.0).1, 1);
        assert_eq!(calendar(MONTH * 12.0).0, 2);
        assert_eq!(calendar(MONTH * 13.5).1, 2);
        assert!((calendar(MONTH * 13.5).3 - 0.5).abs() < 1e-4);
    }

    #[test]
    fn only_living_cover_counts_as_vegetation() {
        let mut map = TileMap::filled(4, 4, Tile::new(Ground::SoilLow, Overlay::Grass));
        map.set(0, 0, Tile::new(Ground::SoilLow, Overlay::Infernal));
        map.set(1, 0, Tile::bare(Ground::Sand));
        map.set(2, 0, Tile::new(Ground::SoilLow, Overlay::Blight));
        map.set(3, 0, Tile::new(Ground::SoilLow, Overlay::Void));
        map.set(0, 1, Tile::new(Ground::SoilLow, Overlay::Candy));
        map.set(
            1,
            1,
            Tile::new(Ground::SoilHigh, Overlay::Maple).with_flora(Some(Flora::Tree)),
        );

        assert_eq!(greenery(&map), 12);
    }
}
