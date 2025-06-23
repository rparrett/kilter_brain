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
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .id();

    let container = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            BorderRadius::bottom(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        column_gap: Val::Px(5.),
                        ..default()
                    },
                    ClimbInfo,
                    Interaction::None,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Name".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR_EMPHASIS.into()),
                        ClimbNameText,
                    ));

                    parent.spawn((
                        Text::new("by Author".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR_MUTED.into()),
                        ClimbAuthorText,
                    ));
                });

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(3.),
                        display: Display::None,
                        ..default()
                    },
                    ClimbMoreInfo,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("(No Description)".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR.into()),
                        ClimbDescriptionText,
                    ));
                    parent.spawn((
                        Text::new("Setter Angle: 40°".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR.into()),
                        ClimbAngleText,
                    ));
                    parent.spawn((
                        Text::new("Draft:".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR.into()),
                    ));
                    parent.spawn((
                        Text::new("Listed:".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR.into()),
                        ClimbListedText,
                    ));
                    parent.spawn((
                        Text::new("UUID".to_string()),
                        TextFont {
                            font_size: theme::FONT_SIZE,
                            ..default()
                        },
                        TextColor(theme::FONT_COLOR.into()),
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

    let Ok(name_entity) = climb_name_text_query.single() else {
        return;
    };
    let Ok(mut name_text) = text_query.get_mut(name_entity) else {
        return;
    };
    name_text.0.clone_from(&climb.name);

    let Ok(author_entity) = climb_author_text_query.single() else {
        return;
    };
    let Ok(mut author_text) = text_query.get_mut(author_entity) else {
        return;
    };
    author_text
        .0
        .clone_from(&format!("by {}", &climb.setter_username));

    let Ok(angle_entity) = climb_angle_text_query.single() else {
        return;
    };
    let Ok(mut angle_text) = text_query.get_mut(angle_entity) else {
        return;
    };
    angle_text.0.clone_from(
        &climb
            .angle
            .map(|a| format!("Setter Angle: {}°", a))
            .unwrap_or_else(|| "Setter Angle: Unknown".to_string()),
    );

    let Ok(description_entity) = climb_description_text_query.single() else {
        return;
    };
    let Ok(mut description_text) = text_query.get_mut(description_entity) else {
        return;
    };
    if !climb.description.is_empty() {
        description_text.0.clone_from(&climb.description);
    } else {
        description_text.0 = "No Description".to_string();
    }

    let Ok(uuid_entity) = climb_uuid_text_query.single() else {
        return;
    };
    let Ok(mut uuid_text) = text_query.get_mut(uuid_entity) else {
        return;
    };
    uuid_text.0.clone_from(&climb.uuid);

    let Ok(draft_entity) = climb_draft_text_query.single() else {
        return;
    };
    let Ok(mut draft_text) = text_query.get_mut(draft_entity) else {
        return;
    };
    draft_text
        .0
        .clone_from(&format!("Draft: {:?}", climb.is_draft));

    let Ok(listed_entity) = climb_listed_text_query.single() else {
        return;
    };
    let Ok(mut listed_text) = text_query.get_mut(listed_entity) else {
        return;
    };
    listed_text
        .0
        .clone_from(&format!("Listed: {:?}", climb.is_listed));
}

fn toggle_more_info(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClimbInfo>)>,
    mut more_info_query: Query<&mut Node, With<ClimbMoreInfo>>,
) {
    if interaction_query.iter().any(|i| *i == Interaction::Pressed) {
        if let Ok(mut node) = more_info_query.single_mut() {
            node.display = if node.display == Display::Flex {
                Display::None
            } else {
                Display::Flex
            }
        }
    }
}
