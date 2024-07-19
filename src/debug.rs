use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::kilter_board::KilterSettings;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ResourceInspectorPlugin::<KilterSettings>::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
        ));
    }
}
