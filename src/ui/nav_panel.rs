use bevy::prelude::*;

use crate::{
    kilter_board::ChangeClimbEvent,
    ui::{button::button_bundle, UiAssets},
};

use super::theme;

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

fn setup_nav_panel(mut commands: Commands, handles: Res<UiAssets>) {
    let container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                right: Val::Px(0.),
                column_gap: Val::Px(12.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            BorderRadius::bottom_left(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .id();

    let prev_button = commands
        .spawn((
            button_bundle("\u{E04C}", handles.symbol_font.clone()),
            PrevButton,
        ))
        .id();

    let next_button = commands
        .spawn((
            button_bundle("\u{E04D}", handles.symbol_font.clone()),
            PrevButton,
        ))
        .id();

    commands
        .entity(container)
        .add_children(&[prev_button, next_button]);
}

fn prev_button(
    query: Query<&Interaction, (With<PrevButton>, Changed<Interaction>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        writer.write(ChangeClimbEvent::Prev);
    }
}

fn next_button(
    query: Query<&Interaction, (With<NextButton>, Changed<Interaction>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        writer.write(ChangeClimbEvent::Next);
    }
}
