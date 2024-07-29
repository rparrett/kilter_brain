use bevy::prelude::*;

use crate::{kilter_board::SelectedClimb, kilter_data::KilterData};

use super::theme;

pub struct InfoPanelPlugin;

impl Plugin for InfoPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_info_panel);
        app.add_systems(Update, (update_selected_climb, toggle_more_info));
    }
}

#[derive(Component)]
struct ClimbNameText;
#[derive(Component)]
struct ClimbAuthorText;
#[derive(Component)]
struct ClimbAngleText;
#[derive(Component)]
struct ClimbDescriptionText;
#[derive(Component)]
struct ClimbUuidText;
#[derive(Component)]
struct ClimbDraftText;
#[derive(Component)]
struct ClimbListedText;
#[derive(Component)]
struct ClimbInfo;
#[derive(Component)]
struct ClimbMoreInfo;

fn setup_info_panel(mut commands: Commands) {
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .id();

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            border_radius: BorderRadius::bottom(theme::CONTAINER_BORDER_RADIUS),
            background_color: theme::CONTAINER_BG.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            column_gap: Val::Px(5.),
                            ..default()
                        },
                        ..default()
                    },
                    ClimbInfo,
                    Interaction::None,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Name".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR_EMPHASIS.into(),
                                ..default()
                            },
                        ),
                        ClimbNameText,
                    ));

                    parent.spawn((
                        TextBundle::from_section(
                            "by Author".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR_MUTED.into(),
                                ..default()
                            },
                        ),
                        ClimbAuthorText,
                    ));
                });

            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.),
                            display: Display::None,
                            ..default()
                        },
                        ..default()
                    },
                    ClimbMoreInfo,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "(No Description)".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR.into(),
                                ..default()
                            },
                        ),
                        ClimbDescriptionText,
                    ));
                    parent.spawn((
                        TextBundle::from_section(
                            "Setter Angle: 40°".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR.into(),
                                ..default()
                            },
                        ),
                        ClimbAngleText,
                    ));
                    parent.spawn((
                        TextBundle::from_section(
                            "Draft:".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR.into(),
                                ..default()
                            },
                        ),
                        ClimbDraftText,
                    ));
                    parent.spawn((
                        TextBundle::from_section(
                            "Listed:".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR.into(),
                                ..default()
                            },
                        ),
                        ClimbListedText,
                    ));
                    parent.spawn((
                        TextBundle::from_section(
                            "UUID".to_string(),
                            TextStyle {
                                font_size: theme::FONT_SIZE,
                                color: theme::FONT_COLOR.into(),
                                ..default()
                            },
                        ),
                        ClimbUuidText,
                    ));
                });
        })
        .id();

    commands.entity(root).add_child(container);
}

fn update_selected_climb(
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
    mut text_query: Query<&mut Text>,
    climb_name_text_query: Query<Entity, With<ClimbNameText>>,
    climb_author_text_query: Query<Entity, With<ClimbAuthorText>>,
    climb_angle_text_query: Query<Entity, With<ClimbAngleText>>,
    climb_description_text_query: Query<Entity, With<ClimbDescriptionText>>,
    climb_uuid_text_query: Query<Entity, With<ClimbUuidText>>,
    climb_draft_text_query: Query<Entity, With<ClimbDraftText>>,
    climb_listed_text_query: Query<Entity, With<ClimbListedText>>,
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

    let Ok(name_entity) = climb_name_text_query.get_single() else {
        return;
    };
    let Ok(mut name_text) = text_query.get_mut(name_entity) else {
        return;
    };
    name_text.sections[0].value.clone_from(&climb.name);

    let Ok(author_entity) = climb_author_text_query.get_single() else {
        return;
    };
    let Ok(mut author_text) = text_query.get_mut(author_entity) else {
        return;
    };
    author_text.sections[0]
        .value
        .clone_from(&format!("by {}", &climb.setter_username));

    let Ok(angle_entity) = climb_angle_text_query.get_single() else {
        return;
    };
    let Ok(mut angle_text) = text_query.get_mut(angle_entity) else {
        return;
    };
    angle_text.sections[0].value.clone_from(
        &climb
            .angle
            .map(|a| format!("Setter Angle: {}°", a))
            .unwrap_or_else(|| "Setter Angle: Unknown".to_string()),
    );

    let Ok(description_entity) = climb_description_text_query.get_single() else {
        return;
    };
    let Ok(mut description_text) = text_query.get_mut(description_entity) else {
        return;
    };
    if !climb.description.is_empty() {
        description_text.sections[0]
            .value
            .clone_from(&climb.description);
    } else {
        description_text.sections[0].value = "No Description".to_string();
    }

    let Ok(uuid_entity) = climb_uuid_text_query.get_single() else {
        return;
    };
    let Ok(mut uuid_text) = text_query.get_mut(uuid_entity) else {
        return;
    };
    uuid_text.sections[0].value.clone_from(&climb.uuid);

    let Ok(draft_entity) = climb_draft_text_query.get_single() else {
        return;
    };
    let Ok(mut draft_text) = text_query.get_mut(draft_entity) else {
        return;
    };
    draft_text.sections[0]
        .value
        .clone_from(&format!("Draft: {:?}", climb.is_draft));

    let Ok(listed_entity) = climb_listed_text_query.get_single() else {
        return;
    };
    let Ok(mut listed_text) = text_query.get_mut(listed_entity) else {
        return;
    };
    listed_text.sections[0]
        .value
        .clone_from(&format!("Listed: {:?}", climb.is_listed));

    // text.sections[0].value = format!("{}/{}", selected.0 + 1, kilter.climbs.len());
    // text.sections[2].value.clone_from(&climb.uuid);
    // text.sections[4].value.clone_from(&climb.name);
    // text.sections[6].value.clone_from(&climb.setter_username);
    // text.sections[8].value.clone_from(&climb.description);
    // text.sections[10].value = format!("Angle: {:?}", climb.angle);
    // text.sections[12].value = format!("Draft: {:?}", climb.is_draft);
    // text.sections[14].value = format!("Listed: {:?}", climb.is_listed);
}

fn toggle_more_info(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClimbInfo>)>,
    mut more_info_query: Query<&mut Style, With<ClimbMoreInfo>>,
) {
    if interaction_query.iter().any(|i| *i == Interaction::Pressed) {
        if let Ok(mut style) = more_info_query.get_single_mut() {
            style.display = if style.display == Display::Flex {
                Display::None
            } else {
                Display::Flex
            }
        }
    }
}
