use bevy::{prelude::*, utils::Uuid};

use crate::{
    button::button,
    kilter_data::{Climb, KilterData},
    theme, PlacementIndicator, SelectedClimb,
};

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_buttons_panel, setup_info_panel));
        app.add_systems(Update, (update_selected_climb, clear_button, new_button));
    }
}

#[derive(Component)]
struct ClimbText;
#[derive(Component)]
struct NewButton;
#[derive(Component)]
struct ClearButton;

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
    .take(7)
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

    commands.entity(container).add_child(new_button);
    commands.entity(container).add_child(clear_button);
}

fn update_selected_climb(
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
    mut texts: Query<&mut Text, With<ClimbText>>,
) {
    let Some(climb) = kilter
        .climbs
        .get(&selected.0)
        .or_else(|| kilter.climbs.iter().next().map(|(_id, climb)| climb))
    else {
        return;
    };

    let mut text = texts.single_mut();
    text.sections[0].value.clone_from(&climb.uuid);
    text.sections[2].value.clone_from(&climb.name);
    text.sections[4].value.clone_from(&climb.setter_username);
    text.sections[6].value.clone_from(&climb.description);
    text.sections[8].value = format!("Angle: {:?}", climb.angle);
    text.sections[10].value = format!("Draft: {:?}", climb.is_draft);
    text.sections[12].value = format!("Listed: {:?}", climb.is_listed);
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

        selected.0 = id;
    }
}
