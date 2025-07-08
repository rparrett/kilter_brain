use bevy::prelude::*;
use bevy_simple_text_input::{TextInput, TextInputTextColor, TextInputTextFont, TextInputValue};

use super::theme;

use crate::kilter_board::ChangeClimbEvent;
use crate::kilter_data::{ClimbFilter, KilterData};
use crate::ui::UiAssets;
use crate::ui::list::{ListItemBundles, list};

#[derive(Component)]
struct SearchField;
#[derive(Component)]
struct SearchResultsPanel;
#[derive(Component)]
struct SearchResultItem(String);
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
                row_gap: Val::Px(5.),
                padding: UiRect::vertical(theme::CONTAINER_PADDING.top),
                ..default()
            },
            BorderRadius::left(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    margin: UiRect::horizontal(theme::CONTAINER_PADDING.left),
                    ..default()
                },
                TextInput,
                TextInputTextFont(TextFont {
                    font_size: theme::FONT_SIZE,
                    ..default()
                }),
                TextInputTextColor(theme::FONT_COLOR.into()),
                SearchField,
            ));
            parent.spawn((Node::default(), SearchResultsPanel));
        });
}

fn update_search_results(
    search_field: Query<&TextInputValue, (With<SearchField>, Changed<TextInputValue>)>,
    kilter: Res<KilterData>,
    filter: Res<ClimbFilter>,
    results_panel: Query<Entity, With<SearchResultsPanel>>,
    mut search_panel: Query<&mut Node, With<SearchPanel>>,
    mut commands: Commands,
    handles: Res<UiAssets>,
) {
    let Ok(search_text) = search_field.single() else {
        return;
    };

    let Ok(panel_entity) = results_panel.single() else {
        return;
    };

    let Ok(mut panel_node) = search_panel.single_mut() else {
        return;
    };

    if search_text.0.is_empty() {
        panel_node.display = Display::None;
        return;
    }

    panel_node.display = Display::Flex;

    // Despawn existing search result entities
    commands.entity(panel_entity).despawn_related::<Children>();

    let results = filter
        .filtered_climbs
        .iter()
        .filter_map(|uuid| {
            let c = kilter.climbs.get(uuid)?;
            if c.name
                .to_lowercase()
                .contains(&search_text.0.to_lowercase())
            {
                Some((c.name.clone(), c.uuid.clone()))
            } else {
                None
            }
        })
        .take(15)
        .collect::<Vec<_>>();

    if results.is_empty() {
        return;
    }

    let font_handle = handles.font.clone();
    let list = list(results, move |(_i, result)| ListItemBundles {
        contents: (
            Text::new(result.0),
            TextFont {
                font: font_handle.clone(),
                font_size: theme::FONT_SIZE_SM,
                ..default()
            },
        ),
        container: (SearchResultItem(result.1)),
    });

    commands.entity(panel_entity).with_child(list);
}

fn handle_search_result_click(
    query: Query<(&Interaction, &SearchResultItem), (Changed<Interaction>, With<SearchResultItem>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    for (interaction, item) in &query {
        if *interaction == Interaction::Pressed {
            writer.write(ChangeClimbEvent::SelectByUuid(item.0.clone()));
        }
    }
}
