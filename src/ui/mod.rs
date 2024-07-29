use bevy::prelude::*;

use self::{
    action_panel::ActionPanelPlugin, button::ButtonPlugin, font::FontPlugin,
    info_panel::InfoPanelPlugin, nav_panel::NavPanelPlugin,
};

mod action_panel;
mod button;
mod font;
mod info_panel;
mod nav_panel;
mod theme;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonPlugin,
            InfoPanelPlugin,
            ActionPanelPlugin,
            NavPanelPlugin,
            FontPlugin,
        ));
    }
}
