use bevy::prelude::*;
use bevy_http_client::{prelude::TypedRequest, HttpClient};
use serde::Serialize;
use std::fmt::Write;
use uuid::Uuid;

use crate::{
    gen_api::{GenApiSettings, GeneratedClimb, GeneratedClimbs},
    kilter_board::{BoardAngle, SelectedClimb},
    kilter_data::{Climb, KilterData},
    placement_indicator::PlacementIndicator,
};

use super::{button::button, theme};

pub struct ActionPanelPlugin;

#[derive(Component)]
struct NewButton;
#[derive(Component)]
struct ClearButton;
#[derive(Component)]
struct GenButton;
#[derive(Component)]
struct GenNewButton;
#[derive(Component)]
struct PublishButton;
#[derive(Component)]
struct OpenClimbButton;

#[derive(Serialize)]
struct GenerateRequest {
    prompt: String,
    num: usize,
}

impl Plugin for ActionPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_buttons_panel);
        app.add_systems(
            Update,
            (
                clear_button,
                new_button,
                gen_fill_button,
                gen_new_button,
                publish_button,
                open_climb_button,
            ),
        );
    }
}

fn setup_buttons_panel(mut commands: Commands) {
    let root = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.),
            left: Val::Px(0.),
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .id();

    let container = commands
        .spawn((
            Node {
                padding: theme::CONTAINER_PADDING,
                column_gap: Val::Px(12.),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BorderRadius::top(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .id();

    let new_button = button(&mut commands, "New", NewButton);
    let clear_button = button(&mut commands, "Clear", ClearButton);
    let gen_button = button(&mut commands, "Gen Bill", GenButton);
    let gen_new_button = button(&mut commands, "Gen New", GenNewButton);
    let publish_button = button(&mut commands, "Publish", PublishButton);
    let open_climb_button = button(&mut commands, "Open", OpenClimbButton);

    commands.entity(container).add_children(&[
        new_button,
        clear_button,
        gen_button,
        gen_new_button,
        publish_button,
        open_climb_button,
    ]);

    commands.entity(root).add_child(container);
}

fn clear_button(
    query: Query<&Interaction, (With<ClearButton>, Changed<Interaction>)>,
    mut commands: Commands,
    placement_query: Query<Entity, With<PlacementIndicator>>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        for entity in &placement_query {
            commands.entity(entity).despawn();
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

        selected.0 = kilter.climbs.len() - 1;
    }
}

fn gen_new_button(
    query: Query<&Interaction, (With<GenNewButton>, Changed<Interaction>)>,
    mut ev_request: EventWriter<TypedRequest<GeneratedClimbs>>,
    api_settings: Res<GenApiSettings>,
    angle: Res<BoardAngle>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let request = GenerateRequest {
            prompt: format!("a{}d20", angle.0),
            num: 10,
        };

        ev_request.write(
            HttpClient::new()
                .post(format!("{}/generate", api_settings.host))
                .json(&request)
                .with_type::<GeneratedClimbs>(),
        );
    }
}

fn gen_fill_button(
    query: Query<&Interaction, (With<GenButton>, Changed<Interaction>)>,
    indicator_query: Query<&PlacementIndicator>,
    mut ev_request: EventWriter<TypedRequest<GeneratedClimbs>>,
    api_settings: Res<GenApiSettings>,
    angle: Res<BoardAngle>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let current_frames: String = indicator_query.iter().fold(String::new(), |mut out, ind| {
            let _ = write!(out, "{ind}");
            out
        });

        let request = GenerateRequest {
            prompt: format!("a{}d20{}", angle.0, current_frames),
            num: 10,
        };

        ev_request.write(
            HttpClient::new()
                .post(format!("{}/generate", api_settings.host))
                .json(&request)
                .with_type::<GeneratedClimbs>(),
        );
    }
}

fn publish_button(
    query: Query<&Interaction, (With<PublishButton>, Changed<Interaction>)>,
    indicator_query: Query<&PlacementIndicator>,
    mut ev_request: EventWriter<TypedRequest<GeneratedClimb>>,
    api_settings: Res<GenApiSettings>,
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let current_frames: String = indicator_query.iter().fold(String::new(), |mut out, ind| {
            let _ = write!(out, "{ind}");
            out
        });

        // Get selected or first climb
        let Some((_, climb)) = kilter.climbs.iter().nth(selected.0) else {
            return;
        };

        let mut new_climb = climb.clone();
        new_climb.frames = current_frames;

        ev_request.write(
            HttpClient::new()
                .post(format!("{}/publish", api_settings.host))
                .json(&new_climb)
                .with_type::<GeneratedClimb>(),
        );
    }
}

fn open_climb_button(
    query: Query<&Interaction, (With<OpenClimbButton>, Changed<Interaction>)>,
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let Some((_, climb)) = kilter.climbs.iter().nth(selected.0) else {
            return;
        };

        if let Err(err) =
            webbrowser::open(&format!("https://kilterboardapp.com/climbs/{}", climb.uuid))
        {
            warn!("Failed to open url: {:?}", err);
        }
    }
}
