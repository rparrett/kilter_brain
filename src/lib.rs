use authoring::AuthoringPlugin;
use bevy::prelude::*;
use bevy_simple_text_input::TextInputPlugin;
use clipboard::ClipboardPlugin;
use debug::DebugPlugin;
use gen_api::GenApiPlugin;
use human::HumanPlugin;
use kilter_board::KilterBoardPlugin;
use pan_cam::PanCamPlugin;
use placement_indicator::PlacementIndicatorPlugin;
use ui::UiPlugin;

use crate::{
    board_connection::BoardConnectionPlugin, effects::EffectsPlugin, nav::NavPlugin,
    prefs::PrefsPlugin,
};

mod authoring;
pub mod board_connection;
mod clipboard;
mod debug;
mod effects;
mod gen_api;
mod human;
mod kilter_board;
pub mod kilter_data;
mod nav;
mod pan_cam;
mod placement_indicator;
mod prefs;
mod ui;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Bevy plugins
        app.add_plugins(MeshPickingPlugin);

        // Our plugins
        app.add_plugins((
            GenApiPlugin,
            HumanPlugin,
            AuthoringPlugin,
            ClipboardPlugin,
            PlacementIndicatorPlugin,
            PanCamPlugin,
            DebugPlugin,
            KilterBoardPlugin,
            UiPlugin,
            BoardConnectionPlugin,
            EffectsPlugin,
            NavPlugin,
            PrefsPlugin,
        ));

        // Third-party Plugins
        app.add_plugins(TextInputPlugin);
    }
}
