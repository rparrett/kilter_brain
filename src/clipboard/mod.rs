use bevy::prelude::*;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod native_clipboard;
#[cfg(target_arch = "wasm32")]
mod wasm_clipboard;

#[derive(Event)]
pub struct PasteEvent(pub String);

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PasteEvent>();

        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        app.add_plugins(native_clipboard::NativeClipboardPlugin);
        #[cfg(target_arch = "wasm32")]
        app.add_plugins(wasm_clipboard::WasmClipboardPlugin);
    }
}
