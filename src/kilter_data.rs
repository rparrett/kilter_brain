use bevy::math::FloatOrd;
use bevy::platform::collections::{HashMap, HashSet};
use combine::EasyParser;
use indexmap::{IndexMap, IndexSet};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
#[cfg(not(target_arch = "wasm32"))]
use std::{fs::read_dir, fs::File, io, io::BufReader, path::Path};

use combine::error::ParseError;
use combine::stream::RangeStream;
use combine::{many1, parser::char::digit, Parser};

#[cfg(not(any(target_arch = "wasm32")))]
use rusqlite::{Connection, Result};

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct KilterData {
    pub leds: HashMap<u32, Led>,
    pub holes: HashMap<u32, Hole>,
    pub placements: HashMap<u32, Placement>,
    pub placement_roles: HashMap<u32, PlacementRole>,
    pub climbs: IndexMap<String, Climb>,
    pub placement_id_to_led_position: HashMap<u32, u32>,
    pub uuid_angle_to_stats: HashMap<(String, u32), Stats>,
    pub difficulty_grades: HashMap<u32, DifficultyGrade>,
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct Stats {
    climb_uuid: String,
    angle: u32,
    display_difficulty: f32,
    benchmark_difficulty: Option<f32>,
    pub ascensionist_count: u32,
    difficulty_average: f32,
    pub quality_average: f32,
    fa_username: String,
    fa_at: String,
}

pub struct DifficultyGrade {
    pub difficulty: u32,
    pub boulder_name: String,
    pub route_name: String,
    pub is_listed: bool,
}

impl KilterData {
    #[cfg(not(any(target_arch = "wasm32")))]
    pub fn from_sqlite(path: &str) -> Result<Self> {
        let conn = Connection::open(path).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT
                    id, product_id, name, x, y, mirrored_hole_id, mirror_group
                FROM holes",
            )
            .unwrap();

        let holes = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    Hole {
                        id: row.get(0)?,
                        product_id: row.get(1)?,
                        name: row.get(2)?,
                        x: row.get(3)?,
                        y: row.get(4)?,
                        mirrored_hole_id: row.get(5)?,
                        mirror_group: row.get(6)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT
                    id, layout_id, hole_id, set_id, default_placement_role_id
                FROM placements",
            )
            .unwrap();

        let placements = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    Placement {
                        id: row.get(0)?,
                        layout_id: row.get(1)?,
                        hole_id: row.get(2)?,
                        set_id: row.get(3)?,
                        default_placement_role_id: row.get(4)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT
                    id, product_id, position,name, full_name, led_color, screen_color
                FROM placement_roles",
            )
            .unwrap();

        let placement_roles = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    PlacementRole {
                        id: row.get(0)?,
                        product_id: row.get(1)?,
                        position: row.get(2)?,
                        name: row.get(3)?,
                        full_name: row.get(4)?,
                        led_color: row.get(5)?,
                        screen_color: row.get(6)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT
                    uuid, name, description, hsm,
                    edge_left, edge_right, edge_bottom, edge_top,
                    frames_count, frames_pace, frames, setter_id, setter_username,
                    layout_id, is_draft, is_listed, angle
                FROM climbs
                WHERE layout_id = 1",
            )
            .unwrap();

        let climbs = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    Climb {
                        uuid: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        hsm: row.get(3)?,
                        edge_left: row.get(4)?,
                        edge_right: row.get(5)?,
                        edge_bottom: row.get(6)?,
                        edge_top: row.get(7)?,
                        frames_count: row.get(8)?,
                        frames_pace: row.get(9)?,
                        frames: row.get(10)?,
                        setter_id: row.get(11)?,
                        setter_username: row.get(12)?,
                        layout_id: row.get(13)?,
                        is_draft: row.get(14)?,
                        is_listed: row.get(15)?,
                        angle: row.get(16)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT
                    id, product_size_id, hole_id, position
                FROM leds
                WHERE product_size_id = 10",
            )
            .unwrap();

        let leds = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    Led {
                        id: row.get(0)?,
                        product_size_id: row.get(1)?,
                        hole_id: row.get(2)?,
                        position: row.get(3)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT
                    placements.id,
                    leds.position
                FROM placements
                INNER JOIN leds ON leds.hole_id = placements.hole_id AND leds.product_size_id = 10
                WHERE placements.layout_id = 1",
            )
            .unwrap();

        let placement_id_to_led_position = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .flatten()
            .collect();

        let mut stmt = conn
            .prepare(
                "SELECT
                    climb_uuid,
                    angle,
                    display_difficulty,
                    benchmark_difficulty,
                    ascensionist_count,
                    difficulty_average,
                    quality_average,
                    fa_username,
                    fa_at
                FROM climb_stats",
            )
            .unwrap();

        let uuid_angle_to_stats = stmt
            .query_map([], |row| {
                Ok((
                    (row.get(0)?, row.get(1)?),
                    Stats {
                        climb_uuid: row.get(0)?,
                        angle: row.get(1)?,
                        display_difficulty: row.get(2)?,
                        benchmark_difficulty: row.get(3)?,
                        ascensionist_count: row.get(4)?,
                        difficulty_average: row.get(5)?,
                        quality_average: row.get(6)?,
                        fa_username: row.get(7)?,
                        fa_at: row.get(8)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect::<HashMap<_, _>>();

        let mut stmt = conn
            .prepare(
                "SELECT
                    difficulty, boulder_name, route_name, is_listed
                FROM difficulty_grades",
            )
            .unwrap();

        let difficulty_grades = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    DifficultyGrade {
                        difficulty: row.get(0)?,
                        boulder_name: row.get(1)?,
                        route_name: row.get(2)?,
                        is_listed: row.get(3)?,
                    },
                ))
            })
            .unwrap()
            .flatten()
            .collect();

        Ok(Self {
            holes,
            placements,
            placement_roles,
            climbs,
            leds,
            placement_id_to_led_position,
            uuid_angle_to_stats,
            difficulty_grades,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn json_update_files<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        for file in read_dir(&path)?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|file_name| file_name.ends_with(".json"))
        {
            let file_path = path.as_ref().join(file);
            self.json_update_file(file_path)?;
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn json_update_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.json_update_reader(reader);
        Ok(())
    }

    // pub fn search_by_name(&self, name: &str) -> Vec<(usize, &Climb)> {
    //     self.climbs
    //         .values()
    //         .filter(|climb| climb.name.contains(name))
    //         .map(|climb| climb.uuid.clone())
    //         .collect()
    // }

    pub fn json_update_reader<R: Read>(&mut self, reader: R) {
        let val: Value = serde_json::from_reader(reader).unwrap();

        let puts = val.get("PUT").unwrap();

        if let Some(climbs) = puts.get("climbs") {
            let climbs = climbs.as_array().unwrap();
            for climb_val in climbs {
                let climb: Climb = serde_json::from_value(climb_val.clone()).unwrap();
                if climb.layout_id != 1 {
                    continue;
                }
                self.climbs.insert(climb.uuid.clone(), climb);
            }
        }

        if let Some(placements) = puts.get("placements") {
            let placements = placements.as_array().unwrap();
            for placement_val in placements {
                let placement: Placement = serde_json::from_value(placement_val.clone()).unwrap();
                self.placements.insert(placement.id, placement);
            }
        }

        if let Some(holes) = puts.get("holes") {
            let holes = holes.as_array().unwrap();
            for hole_val in holes {
                let hole: Hole = serde_json::from_value(hole_val.clone()).unwrap();
                self.holes.insert(hole.id, hole);
            }
        }

        if let Some(placement_roles) = puts.get("placement_roles") {
            let placement_roles = placement_roles.as_array().unwrap();
            for placement_role_val in placement_roles {
                let placement_role: PlacementRole =
                    serde_json::from_value(placement_role_val.clone()).unwrap();
                self.placement_roles
                    .insert(placement_role.id, placement_role);
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Hole {
    pub id: u32,
    pub product_id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub mirrored_hole_id: u32,
    pub mirror_group: u32,
}
#[derive(Deserialize, Debug)]
pub struct Placement {
    pub id: u32,
    pub layout_id: u32,
    pub hole_id: u32,
    pub set_id: u32,
    //pub hold_id: u32,
    //pub rotation: u32,
    pub default_placement_role_id: Option<u32>,
}
#[derive(Deserialize, Debug)]
pub struct PlacementRole {
    pub id: u32,
    pub product_id: u32,
    pub position: u32,
    // pub min_count_in_climb: Option<u32>,
    // pub max_count_in_climb: Option<u32>,
    pub name: String,
    pub full_name: String,
    pub led_color: String,
    pub screen_color: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Climb {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub hsm: u32,
    pub edge_left: i32,
    pub edge_right: i32,
    pub edge_bottom: i32,
    pub edge_top: i32,
    pub frames_count: u32,
    pub frames_pace: u32,
    pub frames: String,
    pub setter_id: u32,
    pub setter_username: String,
    pub layout_id: u32,
    pub is_draft: bool,
    pub is_listed: bool,
    pub angle: Option<u32>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Led {
    pub id: u32,
    pub product_size_id: u32,
    pub hole_id: u32,
    pub position: u32,
}

#[derive(Default)]
pub enum ClimbSort {
    #[default]
    Best,
}

#[derive(Resource)]
pub struct ClimbFilter {
    /// The set of climb UUIDs matching the current filters.
    /// TODO this should be made private.
    pub filtered_climbs: IndexSet<String>,
    /// If not empty, only show climbs from this set instead of doing normal filtering.
    pub override_climbs: HashSet<String>,
    pub angle: u32,
    pub filter_min_difficulty: u32,
    pub filter_max_difficulty: u32,
    pub sort: ClimbSort,
}
impl Default for ClimbFilter {
    fn default() -> Self {
        Self {
            filtered_climbs: Default::default(),
            override_climbs: Default::default(),
            angle: Default::default(),
            filter_min_difficulty: 0,
            filter_max_difficulty: 33,
            sort: Default::default(),
        }
    }
}
impl ClimbFilter {
    pub fn new(angle: u32, kilter_data: &KilterData) -> Self {
        let mut cf = Self {
            angle,
            ..Default::default()
        };

        cf.update(kilter_data);

        cf
    }
    pub fn update(&mut self, kilter_data: &KilterData) {
        self.filtered_climbs.clear();

        for (uuid, _climb) in kilter_data.climbs.iter() {
            if !self.override_climbs.is_empty() {
                if self.override_climbs.contains(uuid) {
                    self.filtered_climbs.insert(uuid.clone());
                }

                continue;
            }

            // TODO how can we avoid the `uuid` allocation here?
            // TODO we need to be able to optionally skip difficulty filtering
            // to show "open projects"
            let Some(stats) = kilter_data
                .uuid_angle_to_stats
                .get(&(uuid.clone(), self.angle))
            else {
                continue;
            };

            if stats.display_difficulty < self.filter_min_difficulty as f32
                || stats.display_difficulty > self.filter_max_difficulty as f32
            {
                continue;
            }

            self.filtered_climbs.insert(uuid.clone());
        }

        self.filtered_climbs.sort_by_cached_key(|climb| {
            let (rating, ascents) = kilter_data
                .uuid_angle_to_stats
                .get(&(climb.clone(), self.angle))
                .map(|s| (s.quality_average, s.ascensionist_count))
                .unwrap_or((0.0, 0));
            // TODO global_avg
            FloatOrd(-weighted_rating(rating, ascents, 10, 2.5))
        });
    }
}

fn weighted_rating(avg_rating: f32, num_ratings: u32, min_ratings: u32, global_avg: f32) -> f32 {
    let confidence = num_ratings as f32 / (num_ratings + min_ratings) as f32;
    confidence * avg_rating + (1.0 - confidence) * global_avg
}

// TODO can we parse into a HashMap<u32, u32>?
// TODO this is probably too much unjustified complexity. The format is simple enough
// that we don't really need fancy parse errors.
pub fn placements_and_roles<'a, I>() -> impl Parser<I, Output = Vec<(u32, u32)>>
where
    I: RangeStream<Token = char, Range = &'a str>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let placement_and_role = (
        combine::parser::char::char('p'),
        many1::<String, _, _>(digit()),
        combine::parser::char::char('r'),
        many1::<String, _, _>(digit()),
    )
        .map(|(_, p, _, r)| (p.parse::<u32>().unwrap(), r.parse::<u32>().unwrap()));

    many1(placement_and_role)
}

pub fn parse_placements_and_roles(input: &str) -> Result<Vec<(u32, u32)>, String> {
    match placements_and_roles().easy_parse(combine::stream::position::Stream::new(input)) {
        Ok((output, _remaining_input)) => Ok(output),
        Err(err) => Err(format!("{err}")),
    }
}
