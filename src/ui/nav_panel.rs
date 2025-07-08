use bevy::prelude::*;

use crate::{
    kilter_board::ChangeClimbEvent,
    kilter_data::{ClimbFilter, KilterData},
    ui::{button::button, UiAssets},
};

use super::theme;

const MIN_GRADE: u32 = 10;
const MAX_GRADE: u32 = 27; // Up to 33 is valid

#[derive(Component)]
pub struct PrevButton;
#[derive(Component)]
pub struct NextButton;
#[derive(Component)]
pub struct GradeButton(Option<u32>);
#[derive(Component)]
pub struct OverrideButton;

pub struct NavPanelPlugin;

impl Plugin for NavPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_nav_panel);
        app.add_systems(
            Update,
            (
                prev_button,
                next_button,
                grade_button,
                override_button,
                override_button_style,
            ),
        );
    }
}

fn setup_nav_panel(mut commands: Commands, handles: Res<UiAssets>) {
    let container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                right: Val::Px(0.),
                column_gap: Val::Px(12.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            BorderRadius::bottom_left(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .id();

    let override_button = commands
        .spawn((button("\u{E342}", handles.font.clone()), OverrideButton))
        .id();

    let grade_button = commands
        .spawn((button("Any", handles.font.clone()), GradeButton(None)))
        .id();

    let prev_button = commands
        .spawn((button("\u{E04C}", handles.symbol_font.clone()), PrevButton))
        .id();

    let next_button = commands
        .spawn((button("\u{E04D}", handles.symbol_font.clone()), NextButton))
        .id();

    commands.entity(container).add_children(&[
        override_button,
        grade_button,
        prev_button,
        next_button,
    ]);
}

fn prev_button(
    query: Query<&Interaction, (With<PrevButton>, Changed<Interaction>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        writer.write(ChangeClimbEvent::Prev);
    }
}

fn next_button(
    query: Query<&Interaction, (With<NextButton>, Changed<Interaction>)>,
    mut writer: EventWriter<ChangeClimbEvent>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        writer.write(ChangeClimbEvent::Next);
    }
}

fn grade_button(
    interactions: Query<&Interaction, (With<GradeButton>, Changed<Interaction>)>,
    mut text_query: Query<&mut Text>,
    mut buttons: Query<(&mut GradeButton, &Children)>,
    data: Res<KilterData>,
    mut filter: ResMut<ClimbFilter>,
) {
    if !interactions.iter().any(|i| *i == Interaction::Pressed) {
        return;
    };

    let Ok((mut button, children)) = buttons.single_mut() else {
        return;
    };

    button.0 = match button.0 {
        Some(difficulty) => {
            if difficulty + 1 > MAX_GRADE {
                None
            } else {
                Some(difficulty + 1)
            }
        }
        None => Some(MIN_GRADE),
    };

    let label = match button.0 {
        Some(difficulty) => {
            match data
                .difficulty_grades
                .get(&difficulty)
                .map(|dg| dg.boulder_name.clone())
            {
                Some(l) => l,
                None => {
                    warn!("Failed to look up difficulty: {}", difficulty);
                    return;
                }
            }
        }
        None => "Any".to_string(),
    };

    let mut iter = text_query.iter_many_mut(children);
    while let Some(mut text) = iter.fetch_next() {
        text.0.clone_from(&label);
    }

    filter.filter_min_difficulty = button.0.unwrap_or(0);
    filter.filter_max_difficulty = button.0.unwrap_or(33);
    filter.update(&data);
}

fn override_button(
    query: Query<&Interaction, (With<OverrideButton>, Changed<Interaction>)>,
    mut filter: ResMut<ClimbFilter>,
    data: Res<KilterData>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        filter.override_climbs.clear();
        filter.update(&data);
    }
}

fn override_button_style(
    filter: Res<ClimbFilter>,
    mut buttons: Query<&mut Node, With<OverrideButton>>,
) {
    if !filter.is_changed() {
        return;
    }

    let Ok(mut node) = buttons.single_mut() else {
        return;
    };

    node.display = if filter.override_climbs.is_empty() {
        Display::None
    } else {
        Display::Flex
    }
}
