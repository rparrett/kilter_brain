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

mod authoring;
mod clipboard;
mod debug;
mod gen_api;
mod human;
mod kilter_board;
pub mod kilter_data;
mod pan_cam;
mod placement_indicator;
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
        ));

        // Third-party Plugins
        app.add_plugins(TextInputPlugin);
    }
}
