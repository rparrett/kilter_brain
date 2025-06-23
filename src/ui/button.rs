use bevy::prelude::*;

use super::theme;

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
            Button,
            Node {
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(12.)),
                ..default()
            },
            BorderRadius::all(Val::Px(3.)),
            BackgroundColor(theme::NORMAL_BUTTON.into()),
            marker,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: theme::FONT_SIZE,
                    ..default()
                },
                TextColor(theme::FONT_COLOR.into()),
            ));
        })
        .id()
}
