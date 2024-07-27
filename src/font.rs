use bevy::{asset::load_internal_binary_asset, prelude::*};

pub struct FontPlugin;
impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            Handle::default(),
            "../assets/FiraMono-Medium.ttf",
            |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
        );
    }
}
