use serde_derive::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
#[cfg(not(target_arch = "wasm32"))]
use std::{fs::File, io::BufReader, path::Path};

use combine::error::ParseError;
use combine::stream::RangeStream;
use combine::{many, many1, parser::char::digit, Parser};

#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, Result};

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct KilterData {
    pub holes: HashMap<u32, Hole>,
    pub placements: HashMap<u32, Placement>,
    pub placement_roles: HashMap<u32, PlacementRole>,
    pub climbs: HashMap<String, Climb>,
}

impl KilterData {
    #[cfg(not(target_arch = "wasm32"))]
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

        Ok(Self {
            holes,
            placements,
            placement_roles,
            climbs,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn json_update_file<P: AsRef<Path>>(&mut self, path: P) {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        self.json_update_reader(reader);
    }

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
                self.placements.insert(placement.id.clone(), placement);
            }
        }

        if let Some(holes) = puts.get("holes") {
            let holes = holes.as_array().unwrap();
            for hole_val in holes {
                let hole: Hole = serde_json::from_value(hole_val.clone()).unwrap();
                self.holes.insert(hole.id.clone(), hole);
            }
        }

        if let Some(placement_roles) = puts.get("placement_roles") {
            let placement_roles = placement_roles.as_array().unwrap();
            for placement_role_val in placement_roles {
                let placement_role: PlacementRole =
                    serde_json::from_value(placement_role_val.clone()).unwrap();
                self.placement_roles
                    .insert(placement_role.id.clone(), placement_role);
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Hole {
    pub id: u32,
    pub product_id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub mirrored_hole_id: u32,
    pub mirror_group: u32,
}
#[derive(Deserialize)]
pub struct Placement {
    pub id: u32,
    pub layout_id: u32,
    pub hole_id: u32,
    pub set_id: u32,
    //pub hold_id: u32,
    //pub rotation: u32,
    pub default_placement_role_id: Option<u32>,
}
#[derive(Deserialize)]
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

#[derive(Deserialize, Default)]
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

    many(placement_and_role)
}
