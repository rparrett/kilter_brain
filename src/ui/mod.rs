use bevy::prelude::*;

use self::{
    action_panel::ActionPanelPlugin, board_panel::BoardPanelPlugin, button::ButtonPlugin,
    font::FontPlugin, info_panel::InfoPanelPlugin, nav_panel::NavPanelPlugin,
    net_panel::NetPanelPlugin,
};

mod action_panel;
mod board_panel;
mod button;
mod font;
mod info_panel;
mod nav_panel;
mod net_panel;
mod theme;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonPlugin,
            BoardPanelPlugin,
            InfoPanelPlugin,
            ActionPanelPlugin,
            NavPanelPlugin,
            NetPanelPlugin,
            FontPlugin,
        ));
    }
}
