use bevy::prelude::*;

use crate::kilter_data::KilterData;

pub struct BoardConnectionPlugin;

impl Plugin for BoardConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NearbyBoards>();
        app.init_resource::<BoardConnection>();
        app.add_event::<StartScan>();
        app.add_event::<StopScan>();
        app.add_event::<Connect>();
        app.add_event::<Disconnect>();
        app.add_event::<WriteToBoard>();
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct BoardDevice {
    pub name: String,
    pub id: String,
}

#[derive(Resource, Default, Eq, PartialEq)]
pub struct NearbyBoards(pub Vec<BoardDevice>);

#[derive(Resource, Default, Eq, PartialEq)]
pub struct BoardConnection {
    pub connected: bool,
    pub scanning: bool,
    pub enabled: bool,
}

#[derive(Event, Default)]
pub struct StartScan;

#[derive(Event, Default)]
pub struct StopScan;

#[derive(Event, Default)]
pub struct Disconnect;

#[derive(Event)]
pub struct Connect {
    pub device_id: String,
}

#[derive(Event)]
pub struct WriteToBoard(pub Vec<(u16, (u8, u8, u8))>);

impl WriteToBoard {
    pub fn from_positions_and_roles(pr: &[(u32, u32)], kd: &KilterData) -> Self {
        let positions = pr
            .iter()
            .flat_map(|(placement_id, role_id)| {
                let hex = kd
                    .placement_roles
                    .get(role_id)
                    .map(|pr| &pr.led_color)
                    .or_else(|| {
                        error!("Role lookup failed: {role_id}");
                        None
                    })?;

                let rgb = hex_to_rgb(hex)
                    .map_err(|e| {
                        error!("Error converting color: {e}");
                    })
                    .ok()?;

                let position = kd
                    .placement_id_to_led_position
                    .get(placement_id)
                    .or_else(|| {
                        error!("Position lookup failed: {role_id}");
                        None
                    })?;

                Some((*position as u16, rgb))
            })
            .collect::<Vec<_>>();

        Self(positions)
    }
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), &'static str> {
    if hex.len() != 6 {
        return Err("Hex string must be 6 characters long");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component")?;

    Ok((r, g, b))
}
