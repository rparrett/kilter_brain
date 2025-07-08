use std::env;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::winit::WinitSettings;
use ble::BlePlugin;
use kilter_brain::kilter_data::KilterData;
use kilter_brain::AppPlugin;

mod ble;

#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
extern "C" fn main_rs() {
    main()
}

#[bevy_main]
fn main() {
    let db_path = env::current_exe()
        .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
        .unwrap()
        .join("assets/db.sqlite3");

    let kd = {
        let mut kd = KilterData::from_sqlite(db_path.to_str().unwrap());
        if let Err(ref e) = kd {
            eprintln!("Failed to load JSON updates. {e:?}");
        };
        kd.unwrap()
    };

    App::new()
        .insert_resource(kd)
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        recognize_pinch_gesture: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    filter: "kilter_brain=debug".into(),
                    ..default()
                }),
            AppPlugin,
            BlePlugin,
        ))
        .insert_resource(WinitSettings::mobile())
        .run();
}
