use bevy::prelude::*;
use bevy::window::WindowMode;
use kilter_brain::kilter_data::KilterData;
use kilter_brain::AppPlugin;

#[bevy_main]
fn main() {
    let kd = {
        let mut kd = KilterData::default();
        kd.json_update_reader(std::io::Cursor::new(include_str!("../../minimal.json")));
        kd
    };

    App::new()
        .insert_resource(kd)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    recognize_pinch_gesture: true,
                    ..default()
                }),
                ..default()
            }),
            AppPlugin,
        ))
        .run();
}
