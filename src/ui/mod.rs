use bevy::prelude::*;

use crate::ui::list::ListPlugin;

use self::{
    action_panel::ActionPanelPlugin, board_panel::BoardPanelPlugin, button::ButtonPlugin,
    font::FontPlugin, info_panel::InfoPanelPlugin, nav_panel::NavPanelPlugin,
    net_panel::NetPanelPlugin, search_panel::SearchPanelPlugin,
};

mod action_panel;
mod board_panel;
mod button;
mod font;
mod info_panel;
mod list;
mod nav_panel;
mod net_panel;
mod search_panel;
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
            SearchPanelPlugin,
            ListPlugin,
        ));
        app.init_resource::<UiAssets>();
    }
}

#[derive(Resource)]
pub struct UiAssets {
    font: Handle<Font>,
    symbol_font: Handle<Font>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            font: asset_server.load("NotoSans-Regular.ttf"),
            symbol_font: asset_server.load("lucide.ttf"),
        }
    }
}
