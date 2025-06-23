use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};

use crate::{gen_api::GenApiSettings, kilter_board::KilterSettings};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            ResourceInspectorPlugin::<KilterSettings>::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
            ResourceInspectorPlugin::<GenApiSettings>::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
        ));
    }
}
