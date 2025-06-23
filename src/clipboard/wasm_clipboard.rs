use bevy::prelude::*;

use std::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use super::PasteEvent;

#[derive(Resource)]
struct OnPasteSender(Mutex<Sender<String>>);

#[derive(Resource)]
struct OnPasteReceiver(Mutex<Receiver<String>>);

pub struct WasmClipboardPlugin;

impl Plugin for WasmClipboardPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = std::sync::mpsc::channel();

        let paste_sender = OnPasteSender(Mutex::new(tx));
        let paste_receiver = OnPasteReceiver(Mutex::new(rx));

        app.add_event::<PasteEvent>()
            .insert_resource(paste_sender)
            .insert_resource(paste_receiver)
            .add_systems(Startup, setup_clipboard_system)
            .add_systems(Update, clipboard);
    }
}

fn setup_clipboard_system(paste_sender: Res<OnPasteSender>) {
    let web_window = web_sys::window().expect("could not get window");
    let local_sender = paste_sender.0.lock().unwrap().clone();

    // TODO this doesn't seem to work when the canvas is focused.

    gloo_events::EventListener::new(&web_window, "paste", move |event| {
        let event = event.dyn_ref::<web_sys::ClipboardEvent>().unwrap_throw();
        if let Some(data) = event.clipboard_data() {
            if let Ok(text) = data.get_data("text") {
                local_sender.write(text.to_owned()).unwrap();
            }
        }
    })
    .forget();
}

fn clipboard(paste_receiver: Res<OnPasteReceiver>, mut events: EventWriter<PasteEvent>) {
    if let Ok(text) = paste_receiver.0.lock().unwrap().try_recv() {
        events.write(PasteEvent(text));
    }
}
