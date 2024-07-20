use bevy::prelude::*;
use clipboard::{ClipboardContext, ClipboardProvider};

use super::PasteEvent;

pub struct NativeClipboardPlugin;

impl Plugin for NativeClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PasteEvent>().add_systems(Update, paste);
    }
}

fn paste(keyboard_input: Res<ButtonInput<KeyCode>>, mut events: EventWriter<PasteEvent>) {
    let ctrl_pressed = keyboard_input.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::SuperLeft,
        KeyCode::ControlRight,
        KeyCode::SuperRight,
    ]);

    if ctrl_pressed && keyboard_input.just_pressed(KeyCode::KeyV) {
        let context: Result<ClipboardContext, _> = ClipboardProvider::new();

        if let Ok(mut context) = context {
            if let Ok(contents) = context.get_contents() {
                events.send(PasteEvent(contents));
            }
        }
    }
}
