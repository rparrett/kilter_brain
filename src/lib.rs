use authoring::AuthoringPlugin;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
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
        ))
        .add_plugins(DefaultPickingPlugins);
    }
}
