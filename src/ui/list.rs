use bevy::{ecs::spawn::SpawnIter, prelude::*};

use crate::ui::theme;

pub struct ListPlugin;
impl Plugin for ListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, interaction);
    }
}

#[derive(Component)]
struct ListItem;

pub struct ListItemBundles<I, O>
where
    I: Bundle,
    O: Bundle,
{
    pub contents: I,
    pub container: O,
}

pub fn list<T, F, B, C, I>(items: I, mapper: F) -> impl Bundle
where
    I: IntoIterator<Item = T>,
    I::IntoIter: Send + Sync + 'static,
    F: Fn((usize, T)) -> ListItemBundles<B, C> + Send + Sync + 'static,
    B: Bundle,
    C: Bundle,
    T: Send,
{
    let iter = items.into_iter().enumerate().map(move |(i, item)| {
        let ListItemBundles {
            contents: inner,
            container: outer,
        } = mapper((i, item));

        (
            ListItem,
            Node {
                padding: theme::LIST_ITEM_PADDING,
                border: if i == 0 {
                    UiRect::all(Val::Px(0.0))
                } else {
                    UiRect::top(Val::Px(1.0))
                },
                ..default()
            },
            Interaction::None,
            BorderColor(theme::LIST_ITEM_BORDER_COLOR.into()),
            outer,
            Children::spawn(Spawn(inner)),
        )
    });
    (
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Children::spawn(SpawnIter(iter)),
    )
}

fn interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ListItem>),
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
                *color = theme::LIST_ITEM_COLOR.into();
            }
        }
    }
}
