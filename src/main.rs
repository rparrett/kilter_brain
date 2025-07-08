use bevy::{log::LogPlugin, prelude::*};

use kilter_brain::{AppPlugin, kilter_data::KilterData};

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
            eprintln!("Failed to load JSON updates. {e:?}");
        };
        if let Err(e) = kd.json_update_files("./assets/json_updates") {
            eprintln!("Failed to load JSON updates. {e:?}");
        };
        kd
    };

    App::new()
        .insert_resource(kd)
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "kilter_brain=debug".into(),
            ..default()
        }))
        .add_plugins(AppPlugin)
        .run();
}
