use bevy::prelude::*;

use crate::{
    kilter_board::{BoardAngle, SelectedClimb},
    kilter_data::KilterData,
    ui::UiAssets,
};

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
struct ClimbRatingText;
#[derive(Component)]
struct ClimbAscentsText;
#[derive(Component)]
struct ClimbInfo;
#[derive(Component)]
struct ClimbMoreInfo;

fn setup_info_panel(mut commands: Commands, handles: Res<UiAssets>) {
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
                        row_gap: Val::Px(5.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ClimbInfo,
                    Interaction::None,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            column_gap: Val::Px(5.),
                            ..default()
                        })
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
                                Text::new("by".to_string()),
                                TextFont {
                                    font_size: theme::FONT_SIZE,
                                    ..default()
                                },
                                TextColor(theme::FONT_COLOR_MUTED.into()),
                            ));

                            parent.spawn((
                                Text::new("Author".to_string()),
                                TextFont {
                                    font_size: theme::FONT_SIZE,
                                    ..default()
                                },
                                TextColor(theme::FONT_COLOR.into()),
                                ClimbAuthorText,
                            ));
                        });
                    parent
                        .spawn(Node {
                            column_gap: Val::Px(5.),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("3.0".to_string()),
                                TextFont {
                                    font_size: theme::FONT_SIZE,
                                    ..default()
                                },
                                TextColor(theme::FONT_COLOR_MUTED.into()),
                                ClimbRatingText,
                            ));

                            parent.spawn((
                                Text::new("\u{E17A}"),
                                TextFont {
                                    font_size: theme::FONT_SIZE_SM,
                                    font: handles.symbol_font.clone(),
                                    ..default()
                                },
                                TextColor(theme::FONT_COLOR_EMPHASIS.into()),
                            ));

                            parent.spawn((
                                Text::new("123".to_string()),
                                TextFont {
                                    font_size: theme::FONT_SIZE,
                                    ..default()
                                },
                                TextColor(theme::FONT_COLOR_MUTED.into()),
                                ClimbAscentsText,
                            ));

                            parent.spawn((
                                Text::new("\u{E1A4}"),
                                TextFont {
                                    font_size: theme::FONT_SIZE_SM,
                                    font: handles.symbol_font.clone(),
                                    ..default()
                                },
                                TextColor(theme::FONT_COLOR.into()),
                            ));
                        });
                });

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(3.),
                        display: Display::None,
                        margin: UiRect::top(Val::Px(5.0)),
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
                        ClimbDraftText,
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
    angle: Res<BoardAngle>,
    mut text_query: Query<&mut Text>,
    climb_name_text_query: Query<Entity, With<ClimbNameText>>,
    climb_author_text_query: Query<Entity, With<ClimbAuthorText>>,
    climb_angle_text_query: Query<Entity, With<ClimbAngleText>>,
    climb_description_text_query: Query<Entity, With<ClimbDescriptionText>>,
    climb_uuid_text_query: Query<Entity, With<ClimbUuidText>>,
    climb_draft_text_query: Query<Entity, With<ClimbDraftText>>,
    climb_listed_text_query: Query<Entity, With<ClimbListedText>>,
    climb_rating_text_query: Query<Entity, With<ClimbRatingText>>,
    climb_ascents_text_query: Query<Entity, With<ClimbAscentsText>>,
) {
    let Some(climb) = kilter.climbs.get(&selected.0) else {
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
    author_text.0.clone_from(&climb.setter_username);

    let Ok(angle_entity) = climb_angle_text_query.single() else {
        return;
    };
    let Ok(mut angle_text) = text_query.get_mut(angle_entity) else {
        return;
    };
    angle_text.0.clone_from(
        &climb
            .angle
            .map(|a| format!("Setter Angle: {a}°"))
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

    let Ok(rating_entity) = climb_rating_text_query.single() else {
        return;
    };
    let Ok(mut rating_text) = text_query.get_mut(rating_entity) else {
        return;
    };

    let stats = kilter
        .uuid_angle_to_stats
        .get(&(selected.0.clone(), angle.0));

    let rating = match stats {
        Some(s) => format!("{:.1}", s.quality_average),
        None => "?".to_string(),
    };

    rating_text.0.clone_from(&rating);

    let Ok(ascents_entity) = climb_ascents_text_query.single() else {
        return;
    };
    let Ok(mut ascents_text) = text_query.get_mut(ascents_entity) else {
        return;
    };

    let ascents = match stats {
        Some(s) => format!("{}", s.ascensionist_count),
        None => "?".to_string(),
    };

    ascents_text.0.clone_from(&ascents);
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
