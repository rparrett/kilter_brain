use bevy::prelude::*;
use bevy_http_client::{HttpClient, prelude::TypedRequest};
use names::Generator;
use serde::Serialize;
use std::fmt::Write;
use uuid::Uuid;

use crate::{
    effects::PartyMode,
    gen_api::{GenApiSettings, GeneratedClimb, GeneratedClimbs},
    kilter_board::{BoardAngle, SelectedClimb},
    kilter_data::{Climb, ClimbFilter, KilterData},
    placement_indicator::PlacementIndicator,
    prefs::UserClimbs,
    ui::{UiAssets, button::button},
};

use super::theme;

pub struct ActionPanelPlugin;

#[derive(Component)]
struct NewButton;
#[derive(Component)]
struct SaveButton;
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
#[derive(Component)]
struct PartyModeButton;

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
                save_button,
                gen_fill_button,
                gen_new_button,
                publish_button,
                open_climb_button,
                party_mode_button,
            ),
        );
    }
}

fn setup_buttons_panel(mut commands: Commands, handles: Res<UiAssets>) {
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

    let spacer = (
        Name::new("Spacer"),
        Node {
            padding: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        },
    );

    let new_button = (button("\u{E644}", handles.font.clone()), NewButton);
    let save_button = (button("\u{E151}", handles.symbol_font.clone()), SaveButton);
    let clear_button = (button("\u{E28F}", handles.font.clone()), ClearButton);
    let gen_button = (button("Gen Fill", handles.font.clone()), GenButton);
    let gen_new_button = (button("Gen New", handles.font.clone()), GenNewButton);
    let publish_button = (button("Publish", handles.font.clone()), PublishButton);
    let open_climb_button = (button("Open", handles.font.clone()), OpenClimbButton);
    let party_mode_button = (
        button("\u{E347}", handles.symbol_font.clone()),
        PartyModeButton,
    );

    commands.entity(container).with_children(|parent| {
        parent.spawn(new_button);
        parent.spawn(save_button);
        parent.spawn(clear_button);
        parent.spawn(spacer.clone());
        parent.spawn(gen_button);
        parent.spawn(gen_new_button);
        parent.spawn(spacer.clone());
        parent.spawn(publish_button);
        parent.spawn(open_climb_button);
        parent.spawn(spacer.clone());
        parent.spawn(party_mode_button);
    });
    commands.entity(root).add_child(container);
}

fn clear_button(
    query: Query<&Interaction, (With<ClearButton>, Changed<Interaction>)>,
    mut commands: Commands,
    placement_query: Query<Entity, With<PlacementIndicator>>,
    mut kilter: ResMut<KilterData>,
    selected: Res<SelectedClimb>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        for entity in &placement_query {
            commands.entity(entity).despawn();
        }

        if let Some(climb) = kilter.climbs.get_mut(&selected.0) {
            climb.frames.clear();
        }
    }
}

fn new_button(
    query: Query<&Interaction, (With<NewButton>, Changed<Interaction>)>,
    mut kilter: ResMut<KilterData>,
    mut filter: ResMut<ClimbFilter>,
    mut selected: ResMut<SelectedClimb>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let id = Uuid::new_v4().to_string();

        let mut generator = Generator::with_naming(names::Name::Numbered);

        kilter.climbs.insert(
            id.clone(),
            Climb {
                uuid: id.clone(),
                setter_username: "User".to_string(),
                name: generator.next().unwrap(),
                ..default()
            },
        );

        filter.override_climbs.clear();
        filter.override_climbs.insert(id.clone());
        filter.update(&kilter);

        selected.0 = id.clone();
    }
}

fn save_button(
    query: Query<&Interaction, (With<SaveButton>, Changed<Interaction>)>,
    mut user_climbs: ResMut<UserClimbs>,
    kilter: Res<KilterData>,
    selected: Res<SelectedClimb>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let Some(climb) = kilter.climbs.get(&selected.0) else {
            return;
        };

        user_climbs.0.insert(climb.uuid.clone(), climb.clone());
    }
}

fn gen_new_button(
    query: Query<&Interaction, (With<GenNewButton>, Changed<Interaction>)>,
    mut ev_request: EventWriter<TypedRequest<GeneratedClimbs>>,
    api_settings: Res<GenApiSettings>,
    angle: Res<BoardAngle>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let request_data = GenerateRequest {
            prompt: format!("a{}d20", angle.0),
            num: 10,
        };

        let request = match HttpClient::new()
            .post(format!("{}/generate", api_settings.host))
            .json(&request_data)
            .try_with_type::<GeneratedClimbs>()
        {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to build request: {e}");
                return;
            }
        };

        ev_request.write(request);
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

        let request_data = GenerateRequest {
            prompt: format!("a{}d20{}", angle.0, current_frames),
            num: 10,
        };

        let request = match HttpClient::new()
            .post(format!("{}/generate", api_settings.host))
            .json(&request_data)
            .try_with_type::<GeneratedClimbs>()
        {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to build request: {e}");
                return;
            }
        };

        ev_request.write(request);
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

        let Some(climb) = kilter.climbs.get(&selected.0) else {
            return;
        };

        let mut new_climb = climb.clone();
        new_climb.frames = current_frames;

        let request = match HttpClient::new()
            .post(format!("{}/publish", api_settings.host))
            .json(&new_climb)
            .try_with_type::<GeneratedClimb>()
        {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to build request: {e}");
                return;
            }
        };

        ev_request.write(request);
    }
}

fn open_climb_button(
    query: Query<&Interaction, (With<OpenClimbButton>, Changed<Interaction>)>,
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        let Some(climb) = kilter.climbs.get(&selected.0) else {
            return;
        };

        if let Err(err) =
            webbrowser::open(&format!("https://kilterboardapp.com/climbs/{}", climb.uuid))
        {
            warn!("Failed to open url: {:?}", err);
        }
    }
}

fn party_mode_button(
    query: Query<&Interaction, (With<PartyModeButton>, Changed<Interaction>)>,
    mut party_mode: ResMut<PartyMode>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        party_mode.0 = !party_mode.0;
    }
}
