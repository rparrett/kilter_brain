use crate::theme;
use bevy::prelude::*;

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, interaction);
    }
}

fn interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = theme::PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = theme::HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = theme::NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn button<M: Component>(commands: &mut Commands, text: &str, marker: M) -> Entity {
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    min_width: Val::Percent(100.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(12.)),
                    ..default()
                },
                background_color: theme::NORMAL_BUTTON.into(),
                ..default()
            },
            marker,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text.to_string(),
                TextStyle {
                    font_size: theme::FONT_SIZE,
                    color: theme::FONT_COLOR,
                    ..default()
                },
            ));
        })
        .id()
}
