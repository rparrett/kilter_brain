use bevy::prelude::*;

use crate::kilter_board::BoardAngle;

use super::{button::button, theme};

#[derive(Component)]
pub struct AngleButton;

pub struct BoardPanelPlugin;

impl Plugin for BoardPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_nav_panel);
        app.add_systems(Update, (angle_button, angle_button_text));
    }
}

fn setup_nav_panel(mut commands: Commands) {
    let container = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                column_gap: Val::Px(12.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            border_radius: BorderRadius::bottom_right(theme::CONTAINER_BORDER_RADIUS),
            background_color: theme::CONTAINER_BG.into(),
            ..default()
        })
        .id();

    let angle_button = button(&mut commands, "0°", AngleButton);

    commands.entity(container).add_child(angle_button);
}

fn angle_button(
    query: Query<&Interaction, (With<AngleButton>, Changed<Interaction>)>,
    mut angle: ResMut<BoardAngle>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        *angle = angle.next();
    }
}

fn angle_button_text(
    angle: Res<BoardAngle>,
    mut text_query: Query<&mut Text>,
    button: Query<&Children, With<AngleButton>>,
) {
    let Ok(children) = button.get_single() else {
        return;
    };
    let mut iter = text_query.iter_many_mut(children);
    while let Some(mut text) = iter.fetch_next() {
        text.sections[0].value = format!("{}°", angle.0);
    }
}
