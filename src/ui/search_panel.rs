use bevy::prelude::*;
use bevy_simple_text_input::{TextInput, TextInputTextColor, TextInputTextFont, TextInputValue};

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
            Node {
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                top: Val::Px(60.),
                right: Val::Px(0.),
                width: Val::Px(200.),
                padding: theme::CONTAINER_PADDING,
                row_gap: Val::Px(5.),
                ..default()
            },
            BorderRadius::left(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node::default(),
                TextInput,
                TextInputTextFont(TextFont {
                    font_size: theme::FONT_SIZE,
                    ..default()
                }),
                TextInputTextColor(theme::FONT_COLOR.into()),
                SearchField,
            ));
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.),
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
    mut search_panel: Query<&mut Node, With<SearchPanel>>,
    mut commands: Commands,
) {
    let Ok(search_text) = search_field.get_single() else {
        return;
    };

    let Ok(panel_entity) = results_panel.get_single() else {
        return;
    };

    let Ok(mut panel_node) = search_panel.get_single_mut() else {
        return;
    };

    if search_text.0.is_empty() {
        panel_node.display = Display::None;
        return;
    }

    panel_node.display = Display::Flex;

    // Despawn existing search result entities
    commands.entity(panel_entity).despawn_descendants();

    let results = kilter.search_by_name(&search_text.0);
    if results.is_empty() {
        return;
    }

    for (climb_idx, climb) in results.iter().take(10) {
        let result = commands
            .spawn((
                Button,
                Node {
                    width: Val::Percent(100.),
                    padding: theme::CONTAINER_PADDING,
                    ..default()
                },
                BackgroundColor(theme::CONTAINER_BG.into()),
                SearchResultItem(*climb_idx),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("{}: {}", climb_idx, climb.name)),
                    TextFont {
                        font_size: theme::FONT_SIZE_SM,
                        ..default()
                    },
                    TextColor(theme::FONT_COLOR.into()),
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
