use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_http_client::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use serde_derive::Deserialize;

use crate::{
    kilter_board::SelectedClimb,
    kilter_data::{Climb, KilterData},
};

pub struct GenApiPlugin;

impl Plugin for GenApiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HttpClientPlugin)
            .add_plugins(
                ResourceInspectorPlugin::<GenApiSettings>::default()
                    .run_if(input_toggle_active(false, KeyCode::Escape)),
            )
            .init_resource::<GenApiSettings>()
            .register_type::<GenApiSettings>()
            .register_request_type::<GeneratedClimbs>()
            // Used by publish button, for now
            .register_request_type::<GeneratedClimb>()
            .add_systems(Update, handle_response);
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct GenApiSettings {
    pub host: String,
}
impl Default for GenApiSettings {
    fn default() -> Self {
        Self {
            //host: "https://www.robparrett.com:5002".to_string(),
            host: "http://localhost:5001".to_string(),
        }
    }
}

pub type GeneratedClimbs = Vec<GeneratedClimb>;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeneratedClimb {
    pub uuid: String,
    pub angle: Option<u32>,
    pub description: String,
    pub difficulty: String,
    pub frames: String,
    pub name: String,
}

fn handle_response(
    mut ev_response: EventReader<TypedResponse<GeneratedClimbs>>,
    mut kilter: ResMut<KilterData>,
    mut selected: ResMut<SelectedClimb>,
) {
    for response in ev_response.read() {
        for generated_climb in &**response {
            kilter.climbs.insert(
                generated_climb.uuid.clone(),
                Climb {
                    uuid: generated_climb.uuid.clone(),
                    setter_username: "API".to_string(),
                    name: generated_climb.name.clone(),
                    frames: generated_climb.frames.clone(),
                    description: generated_climb.description.clone(),
                    angle: generated_climb.angle,
                    ..default()
                },
            );
        }

        selected.0 = kilter.climbs.len() - response.len();
    }
}
