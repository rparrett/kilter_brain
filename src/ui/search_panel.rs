use bevy::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputValue};

use super::theme;

use crate::kilter_board::ChangeClimbEvent;
use crate::kilter_data::KilterData;

#[derive(Component)]
struct SearchField;
#[derive(Component)]
struct SearchResultsPanel;
#[derive(Component)]
struct SearchResultItem(usize);
#[derive(Component)]
struct SearchPanel;

pub struct SearchPanelPlugin;

impl Plugin for SearchPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_search_ui)
            .add_systems(Update, (update_search_results, handle_search_result_click));
    }
}

fn setup_search_ui(mut commands: Commands) {
    commands
        .spawn((
            Name::new("SearchPanel"),
            SearchPanel,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    top: Val::Px(60.),
                    right: Val::Px(0.),
                    width: Val::Px(200.),
                    padding: theme::CONTAINER_PADDING,
                    row_gap: Val::Px(5.),
                    ..default()
                },
                border_radius: BorderRadius::left(theme::CONTAINER_BORDER_RADIUS),
                background_color: theme::CONTAINER_BG.into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle::default(),
                TextInputBundle::default().with_text_style(TextStyle {
                    font_size: theme::FONT_SIZE,
                    color: theme::FONT_COLOR.into(),
                    ..default()
                }),
                SearchField,
            ));
            parent.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.),
                        ..default()
                    },
                    ..default()
                },
                SearchResultsPanel,
            ));
        });
}

fn update_search_results(
    search_field: Query<&TextInputValue, (With<SearchField>, Changed<TextInputValue>)>,
    kilter: Res<KilterData>,
    results_panel: Query<Entity, With<SearchResultsPanel>>,
    mut search_panel: Query<&mut Style, With<SearchPanel>>,
    mut commands: Commands,
) {
    let Ok(search_text) = search_field.get_single() else {
        return;
    };

    let Ok(panel_entity) = results_panel.get_single() else {
        return;
    };

    let Ok(mut panel_style) = search_panel.get_single_mut() else {
        return;
    };

    if search_text.0.is_empty() {
        panel_style.display = Display::None;
        return;
    }

    panel_style.display = Display::Flex;

    // Despawn existing search result entities
    commands.entity(panel_entity).despawn_descendants();

    let results = kilter.search_by_name(&search_text.0);
    if results.is_empty() {
        return;
    }

    for (climb_idx, climb) in results.iter().take(10) {
        let result = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        padding: theme::CONTAINER_PADDING,
                        ..default()
                    },
                    background_color: theme::CONTAINER_BG.into(),
                    ..default()
                },
                SearchResultItem(*climb_idx),
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    format!("{}: {}", climb_idx, climb.name),
                    TextStyle {
                        font_size: theme::FONT_SIZE_SM,
                        color: theme::FONT_COLOR.into(),
                        ..default()
                    },
                ));
            })
            .id();

        commands.entity(panel_entity).add_child(result);
    }
}

fn handle_search_result_click(
    query: Query<(&Interaction, &SearchResultItem), (Changed<Interaction>, With<SearchResultItem>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    for (interaction, item) in &query {
        if *interaction == Interaction::Pressed {
            writer.send(ChangeClimbEvent::SelectByIndex(item.0));
        }
    }
}
