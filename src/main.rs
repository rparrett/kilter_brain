use authoring::AuthoringPlugin;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

use bevy_mod_picking::prelude::*;
use debug::DebugPlugin;
use gen_api::GenApiPlugin;

use button::ButtonPlugin;
use combine::EasyParser;

use clipboard::{ClipboardPlugin, PasteEvent};
use human::HumanPlugin;
use kilter_board::KilterBoardPlugin;
use kilter_data::KilterData;
use pan_cam::PanCamPlugin;
use panels::PanelsPlugin;
use placement_indicator::PlacementIndicatorPlugin;

mod authoring;
mod button;
#[cfg_attr(not(target_arch = "wasm32"), path = "native_clipboard.rs")]
#[cfg_attr(target_arch = "wasm32", path = "wasm_clipboard.rs")]
mod clipboard;
mod debug;
mod gen_api;
mod human;
mod kilter_board;
mod kilter_data;
mod pan_cam;
mod panels;
mod placement_indicator;
mod theme;

fn main() {
    // Just embed some minimal json on the web for now. In the future we will want to
    // be able to load this data from an API endpoint or perhaps just through Bevy's
    // asset server.
    #[cfg(target_arch = "wasm32")]
    let kd = {
        let mut kd = KilterData::default();
        kd.json_update_reader(std::io::Cursor::new(include_str!("../minimal.json")));
        kd
    };
    #[cfg(not(target_arch = "wasm32"))]
    let kd = {
        let mut kd = KilterData::from_sqlite("../kilter_brain_data/db.sqlite3").unwrap();
        if let Err(e) = kd.json_update_files("../kilter_brain_data/api_json") {
            eprintln!("Failed to load JSON updates. {:?}", e);
        };
        kd
    };

    App::new()
        .insert_resource(kd)
        .add_plugins(DefaultPlugins)
        .add_plugins((
            GenApiPlugin,
            ClipboardPlugin,
            HumanPlugin,
            AuthoringPlugin,
            ButtonPlugin,
            PanelsPlugin,
            PlacementIndicatorPlugin,
            PanCamPlugin,
            DebugPlugin,
            KilterBoardPlugin,
        ))
        .add_plugins(DefaultPickingPlugins)
        .run();
}
