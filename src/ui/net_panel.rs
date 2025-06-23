use bevy::prelude::*;
use bevy_http_client::RequestTask;

use super::theme;

#[derive(Component)]
struct LoadingText;

#[derive(Component)]
struct NetPanel;

pub struct NetPanelPlugin;

impl Plugin for NetPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_net_panel);
        app.add_systems(Update, show_hide);
    }
}

fn setup_net_panel(mut commands: Commands) {
    let container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                right: Val::Px(0.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            BorderRadius::top_left(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
            NetPanel,
        ))
        .id();

    // TODO style
    let label = commands
        .spawn((
            Text::new("Loading..."),
            TextFont {
                font_size: theme::FONT_SIZE,
                ..default()
            },
            TextColor(theme::FONT_COLOR.into()),
            LoadingText,
        ))
        .id();

    commands.entity(container).add_child(label);
}

fn show_hide(
    requests: Query<(), With<RequestTask>>,
    mut query: Query<&mut Visibility, With<NetPanel>>,
) {
    let Ok(mut visibility) = query.single_mut() else {
        return;
    };

    let num = requests.iter().len();

    let new_visibility = if num > 0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    visibility.set_if_neq(new_visibility);
}
