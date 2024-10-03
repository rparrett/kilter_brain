use bevy::prelude::*;

use super::theme;

use crate::kilter_board::ChangeClimbEvent;
use crate::kilter_data::KilterData;

#[derive(Component)]
struct SearchField;
#[derive(Component)]
struct SearchResultsPanel;
#[derive(Component)]
struct SearchResultItem;

pub struct SearchPanelPlugin;

impl Plugin for SearchPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_search_ui).add_systems(
            Update,
            (
                update_search_field,
                update_search_results,
                handle_search_result_click,
            ),
        );
    }
}

fn setup_search_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(60.),
                right: Val::Px(0.),
                width: Val::Px(200.),
                height: Val::Px(30.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            border_radius: BorderRadius::top_left(theme::CONTAINER_BORDER_RADIUS),
            background_color: theme::CONTAINER_BG.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "search",
                    TextStyle {
                        font_size: theme::FONT_SIZE,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                SearchField,
            ));
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(80.),
                    right: Val::Px(0.),
                    width: Val::Px(200.),
                    padding: theme::CONTAINER_PADDING,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                border_radius: BorderRadius::bottom_left(theme::CONTAINER_BORDER_RADIUS),
                background_color: theme::CONTAINER_BG.into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            SearchResultsPanel,
        ))
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(0.),
                    height: Val::Px(0.),
                    ..default()
                },
                ..default()
            });
        });
}

fn update_search_field(
    mut query: Query<&mut Text, With<SearchField>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut text = query.single_mut();
    let shift_pressed =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    for key in keyboard_input.get_just_pressed() {
        if let Some(char) = key_to_char(*key, shift_pressed) {
            if text.sections[0].value.len() < 20 {
                text.sections[0].value.push(char);
            }
        }
    }
    // Handle backspace with repeat
    if keyboard_input.pressed(KeyCode::Backspace) {
        if shift_pressed {
            text.sections[0].value.clear();
        } else if keyboard_input.just_pressed(KeyCode::Backspace)
            || time.elapsed_seconds().fract() < 0.05
        {
            text.sections[0].value.pop();
        }
    }
}

fn update_search_results(
    search_field: Query<&Text, With<SearchField>>,
    kilter: Res<KilterData>,
    mut results_panel: Query<(Entity, &mut Visibility), With<SearchResultsPanel>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    if search_field.is_empty() {
        return;
    }
    let Ok(search_text) = search_field
        .get_single()
        .map(|text| &text.sections[0].value)
    else {
        return;
    };
    let Ok((panel_entity, mut visibility)) = results_panel.get_single_mut() else {
        return;
    };

    if search_text.len() >= 3 {
        *visibility = Visibility::Visible;

        // Despawn existing children
        if let Ok(children) = children_query.get(panel_entity) {
            for &child in children.iter() {
                commands.entity(child).despawn_recursive();
            }
        }
        let results = kilter.search_by_name(search_text); // IndexMap<usize, &Climb>
        if results.is_empty() {
            return;
        }

        for (climb_idx, climb) in results.iter().take(10) {
            let result = commands
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Px(40.),
                            padding: theme::CONTAINER_PADDING,
                            ..default()
                        },
                        background_color: theme::CONTAINER_BG.into(),
                        ..default()
                    },
                    SearchResultItem,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("{}: {}", climb_idx, climb.name.clone()),
                        TextStyle {
                            font_size: theme::FONT_SIZE_SM,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                })
                .id();
            commands.entity(panel_entity).add_child(result);
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn handle_search_result_click(
    query: Query<(&Interaction, &Children), (Changed<Interaction>, With<SearchResultItem>)>,
    text_query: Query<&Text>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    for (interaction, children) in query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(text) = text_query.get(children[0]) {
                println!("Clicked on: {}", text.sections[0].value);
                let climb_id = text.sections[0]
                    .value
                    .split(":")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                writer.send(ChangeClimbEvent::SelectByIndex(climb_id));
            }
        }
    }
}

fn key_to_char(key: KeyCode, shift_pressed: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift_pressed { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift_pressed { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift_pressed { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift_pressed { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift_pressed { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift_pressed { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift_pressed { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift_pressed { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift_pressed { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift_pressed { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift_pressed { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift_pressed { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift_pressed { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift_pressed { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift_pressed { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift_pressed { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift_pressed { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift_pressed { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift_pressed { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift_pressed { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift_pressed { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift_pressed { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift_pressed { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift_pressed { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift_pressed { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift_pressed { 'Z' } else { 'z' }),
        KeyCode::Space => Some(' '),
        KeyCode::Semicolon => Some(if shift_pressed { ':' } else { ';' }),
        KeyCode::Digit1 => Some(if shift_pressed { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift_pressed { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift_pressed { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift_pressed { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift_pressed { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift_pressed { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift_pressed { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift_pressed { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift_pressed { '(' } else { '9' }),
        KeyCode::Digit0 => Some(if shift_pressed { ')' } else { '0' }),
        KeyCode::Minus => Some(if shift_pressed { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift_pressed { '+' } else { '=' }),
        _ => None,
    }
}
