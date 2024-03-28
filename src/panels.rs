use std::fmt::Write;

use bevy::{prelude::*, utils::Uuid};

use bevy_http_client::prelude::*;

use crate::{
    button::button,
    gen_api::{GenApiSettings, GeneratedClimb},
    kilter_data::{Climb, KilterData},
    placement_indicator::PlacementIndicator,
    theme, SelectedClimb,
};

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_buttons_panel, setup_info_panel));
        app.add_systems(
            Update,
            (
                update_selected_climb,
                clear_button,
                new_button,
                gen_button,
                gen_new_button,
            ),
        );
    }
}

#[derive(Component)]
struct ClimbText;
#[derive(Component)]
struct NewButton;
#[derive(Component)]
struct ClearButton;
#[derive(Component)]
struct GenButton;
#[derive(Component)]
struct GenNewButton;

fn setup_info_panel(mut commands: Commands) {
    let container = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                left: Val::Px(0.),
                padding: UiRect::all(Val::Px(12.)),
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.3).into(),
            ..default()
        })
        .id();

    let sections = std::iter::repeat_with(|| {
        [
            TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font_size: theme::FONT_SIZE,
                    color: theme::FONT_COLOR,
                    ..default()
                },
            },
            TextSection {
                value: "\n".to_string(),
                style: TextStyle {
                    font_size: theme::FONT_SIZE,
                    color: theme::FONT_COLOR,
                    ..default()
                },
            },
        ]
    })
    .take(8)
    .flatten()
    .collect::<Vec<_>>();

    let text = commands
        .spawn((TextBundle::from_sections(sections), ClimbText))
        .id();

    commands.entity(container).add_child(text);
}

fn setup_buttons_panel(mut commands: Commands) {
    let container = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.),
                right: Val::Px(0.),
                padding: UiRect::all(Val::Px(12.)),
                row_gap: Val::Px(12.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.3).into(),
            ..default()
        })
        .id();

    let new_button = button(&mut commands, "New Climb", NewButton);
    let clear_button = button(&mut commands, "Clear", ClearButton);
    let gen_button = button(&mut commands, "Generate", GenButton);
    let gen_new_button = button(&mut commands, "Gen New", GenNewButton);

    commands.entity(container).add_child(new_button);
    commands.entity(container).add_child(clear_button);
    commands.entity(container).add_child(gen_button);
    commands.entity(container).add_child(gen_new_button);
}

fn update_selected_climb(
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
    mut texts: Query<&mut Text, With<ClimbText>>,
) {
    let Some(climb) = kilter
        .climbs
        .iter()
        .nth(selected.0)
        .or_else(|| kilter.climbs.iter().next())
        .map(|(_, climb)| climb)
    else {
        return;
    };

    let mut text = texts.single_mut();
    text.sections[0].value = format!("{}/{}", selected.0 + 1, kilter.climbs.len());
    text.sections[2].value.clone_from(&climb.uuid);
    text.sections[4].value.clone_from(&climb.name);
    text.sections[6].value.clone_from(&climb.setter_username);
    text.sections[8].value.clone_from(&climb.description);
    text.sections[10].value = format!("Angle: {:?}", climb.angle);
    text.sections[12].value = format!("Draft: {:?}", climb.is_draft);
    text.sections[14].value = format!("Listed: {:?}", climb.is_listed);
}

fn clear_button(
    query: Query<&Interaction, (With<ClearButton>, Changed<Interaction>)>,
    mut commands: Commands,
    placement_query: Query<Entity, With<PlacementIndicator>>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        for entity in &placement_query {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn new_button(
    query: Query<&Interaction, (With<NewButton>, Changed<Interaction>)>,
    mut kilter: ResMut<KilterData>,
    mut selected: ResMut<SelectedClimb>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let id = Uuid::new_v4().to_string();

        kilter.climbs.insert(
            id.clone(),
            Climb {
                uuid: id.clone(),
                setter_username: "User".to_string(),
                name: "New Climb".to_string(),
                ..default()
            },
        );

        selected.0 = kilter.climbs.len();
    }
}

fn gen_new_button(
    query: Query<&Interaction, (With<GenNewButton>, Changed<Interaction>)>,
    mut ev_request: EventWriter<TypedRequest<GeneratedClimb>>,
    api_settings: Res<GenApiSettings>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        ev_request.send(
            HttpClient::new()
                .get(format!("{}/generate/a40d15", api_settings.host))
                .with_type::<GeneratedClimb>(),
        );
    }
}

fn gen_button(
    query: Query<&Interaction, (With<GenButton>, Changed<Interaction>)>,
    indicator_query: Query<&PlacementIndicator>,
    mut ev_request: EventWriter<TypedRequest<GeneratedClimb>>,
    api_settings: Res<GenApiSettings>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let current_frames: String = indicator_query.iter().fold(String::new(), |mut out, ind| {
            let _ = write!(out, "{ind}");
            out
        });

        ev_request.send(
            HttpClient::new()
                .get(format!(
                    "{}/generate/a40d15{}",
                    api_settings.host, current_frames
                ))
                .with_type::<GeneratedClimb>(),
        );
    }
}
