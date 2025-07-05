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

#[derive(Eq, PartialEq)]
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
                let color = match *role_id {
                    12 => (255, 0, 0),
                    13 => (0, 255, 0),
                    14 => (0, 0, 255),
                    15 => (255, 0, 255),
                    _ => (0, 0, 0),
                };
                let Some(position) = kd.placement_id_to_led_position.get(placement_id) else {
                    return None;
                };
                Some((*position as u16, color))
            })
            .collect::<Vec<_>>();

        Self(positions)
    }
}
