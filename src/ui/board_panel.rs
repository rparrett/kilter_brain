use bevy::prelude::*;

use crate::{
    board_connection::{BoardConnection, Connect, Disconnect, NearbyBoards, StartScan, StopScan},
    kilter_board::BoardAngle,
    ui::{button::button_bundle, UiAssets},
};

use super::theme;

#[derive(Component)]
pub struct AngleButton;

#[derive(Component)]
pub struct ScanButton;

#[derive(Component)]
pub struct ConnectButton(String);

#[derive(Component)]
pub struct NearbyBoardsPanel;

pub struct BoardPanelPlugin;

impl Plugin for BoardPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_nav_panel);
        app.add_systems(
            Update,
            (
                angle_button,
                angle_button_text,
                nearby_boards,
                scan_button,
                connect_button,
            ),
        );
    }
}

fn setup_nav_panel(mut commands: Commands, assets: Res<UiAssets>) {
    let container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                column_gap: Val::Px(12.),
                padding: theme::CONTAINER_PADDING,
                ..default()
            },
            BorderRadius::bottom_right(theme::CONTAINER_BORDER_RADIUS),
            BackgroundColor(theme::CONTAINER_BG.into()),
        ))
        .id();

    let angle_button = commands
        .spawn((button_bundle("0°", assets.font.clone()), AngleButton))
        .id();
    let scan_button = commands
        .spawn((
            button_bundle("\u{E060}", assets.symbol_font.clone()),
            ScanButton,
        ))
        .id();

    commands
        .entity(container)
        .add_children(&[angle_button, scan_button]);

    commands.spawn((
        Name::new("NearbyBoardsPanel"),
        NearbyBoardsPanel,
        Node {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            top: Val::Px(60.),
            left: Val::Px(0.),
            width: Val::Px(300.),
            padding: theme::CONTAINER_PADDING,
            row_gap: Val::Px(5.),
            ..default()
        },
        BorderRadius::right(theme::CONTAINER_BORDER_RADIUS),
        BackgroundColor(theme::CONTAINER_BG.into()),
    ));
}

fn angle_button(
    query: Query<&Interaction, (With<AngleButton>, Changed<Interaction>)>,
    mut angle: ResMut<BoardAngle>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        *angle = angle.next();
    }
}

fn angle_button_text(
    angle: Res<BoardAngle>,
    mut text_query: Query<&mut Text>,
    button: Query<&Children, With<AngleButton>>,
) {
    let Ok(children) = button.single() else {
        return;
    };
    let mut iter = text_query.iter_many_mut(children);
    while let Some(mut text) = iter.fetch_next() {
        text.0 = format!("{}°", angle.0);
    }
}

fn scan_button(
    query: Query<&Interaction, (With<ScanButton>, Changed<Interaction>)>,
    board_connection: Res<BoardConnection>,
    mut start_scan_events: EventWriter<StartScan>,
    mut stop_scan_events: EventWriter<StopScan>,
    mut disconnect_events: EventWriter<Disconnect>,
) {
    if query.iter().any(|i| *i == Interaction::Pressed) {
        if board_connection.connected {
            disconnect_events.write_default();
        } else if board_connection.scanning {
            stop_scan_events.write_default();
        } else {
            start_scan_events.write_default();
        }
    }
}

fn nearby_boards(
    mut commands: Commands,
    nearby_boards: Res<NearbyBoards>,
    board_connection: Res<BoardConnection>,
    mut panels: Query<(Entity, &mut Node), With<NearbyBoardsPanel>>,
) {
    if !nearby_boards.is_changed() && !board_connection.is_changed() {
        return;
    }

    let Ok((entity, mut node)) = panels.single_mut() else {
        return;
    };

    node.display =
        if nearby_boards.0.len() > 0 && board_connection.scanning && !board_connection.connected {
            Display::Block
        } else {
            Display::None
        };

    commands.entity(entity).despawn_related::<Children>();

    commands.entity(entity).with_children(|parent| {
        for board in &nearby_boards.0 {
            parent.spawn((
                Button,
                ConnectButton(board.id.clone()),
                Node::default(),
                BackgroundColor(theme::CONTAINER_BG.into()),
                children![Text::new(&board.name)],
            ));
        }
    });
}

fn connect_button(
    query: Query<(&ConnectButton, &Interaction), Changed<Interaction>>,
    mut events: EventWriter<Connect>,
) {
    for (button, interaction) in &query {
        if *interaction == Interaction::Pressed {
            events.write(Connect {
                device_id: button.0.clone(),
            });
        }
    }
}
