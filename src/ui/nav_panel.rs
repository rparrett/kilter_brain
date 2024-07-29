use bevy::prelude::*;

use crate::kilter_board::ChangeClimbEvent;

use super::{button::button, theme};

#[derive(Component)]
pub struct PrevButton;
#[derive(Component)]
pub struct NextButton;

pub struct NavPanelPlugin;

impl Plugin for NavPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_nav_panel);
        app.add_systems(Update, (prev_button, next_button));
    }
}

fn setup_nav_panel(mut commands: Commands) {
    let container = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                right: Val::Px(0.),
                column_gap: Val::Px(12.),
                padding: UiRect::all(Val::Px(12.)),
                ..default()
            },
            border_radius: BorderRadius::bottom_left(Val::Px(10.)),
            background_color: theme::CONTAINER_BG.into(),
            ..default()
        })
        .id();

    let prev_button = button(&mut commands, "←", PrevButton);
    let next_button = button(&mut commands, "→", NextButton);

    commands
        .entity(container)
        .push_children(&[prev_button, next_button]);
}

fn prev_button(
    query: Query<&Interaction, (With<PrevButton>, Changed<Interaction>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        info!("!");
        writer.send(ChangeClimbEvent::Prev);
    }
}

fn next_button(
    query: Query<&Interaction, (With<NextButton>, Changed<Interaction>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        writer.send(ChangeClimbEvent::Next);
    }
}
