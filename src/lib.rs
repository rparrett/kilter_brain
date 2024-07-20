use authoring::AuthoringPlugin;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use button::ButtonPlugin;
use clipboard::ClipboardPlugin;
use debug::DebugPlugin;
use gen_api::GenApiPlugin;
use human::HumanPlugin;
use kilter_board::KilterBoardPlugin;
use pan_cam::PanCamPlugin;
use panels::PanelsPlugin;
use placement_indicator::PlacementIndicatorPlugin;

mod authoring;
mod button;
mod debug;
mod gen_api;
mod human;
mod kilter_board;
mod clipboard;
pub mod kilter_data;
mod pan_cam;
mod panels;
mod placement_indicator;
mod theme;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GenApiPlugin,
            HumanPlugin,
            AuthoringPlugin,
            ClipboardPlugin,
            ButtonPlugin,
            PanelsPlugin,
            PlacementIndicatorPlugin,
            PanCamPlugin,
            DebugPlugin,
            KilterBoardPlugin,
        ))
        .add_plugins(DefaultPickingPlugins);
    }
}
