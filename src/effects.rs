use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    board_connection::WriteToBoard, kilter_data::KilterData,
    placement_indicator::PlacementIndicator,
};

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, party_mode);
        app.init_resource::<PartyMode>();
        app.init_resource::<PartyModeTimer>();
    }
}

#[derive(Resource, Default)]
pub struct PartyMode(pub bool);
#[derive(Resource)]
pub struct PartyModeTimer(Timer);
impl Default for PartyModeTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

/// Illuminates the holds of the selected climb, with each hold given a random color, changing
/// every second.
fn party_mode(
    time: Res<Time>,
    mut timer: ResMut<PartyModeTimer>,
    query: Query<&PlacementIndicator>,
    mut events: EventWriter<WriteToBoard>,
    kd: Res<KilterData>,
    party_mode: Res<PartyMode>,
) {
    if !party_mode.0 {
        return;
    }

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let data = query
        .iter()
        .flat_map(|pr| {
            let color = BRIGHT_COLORS.choose(&mut rand::thread_rng()).unwrap();
            let position = kd.placement_id_to_led_position.get(&pr.placement_id)?;

            Some((*position as u16, *color))
        })
        .collect::<Vec<_>>();

    events.write(WriteToBoard(data));
}

const BRIGHT_COLORS: &[(u8, u8, u8)] = &[
    (0, 182, 0),
    (0, 182, 85),
    (0, 182, 170),
    (0, 182, 255),
    (0, 219, 0),
    (0, 219, 85),
    (0, 219, 170),
    (0, 219, 255),
    (0, 255, 0),
    (0, 255, 85),
    (0, 255, 170),
    (0, 255, 255),
    (36, 146, 255),
    (36, 182, 0),
    (36, 182, 85),
    (36, 182, 170),
    (36, 182, 255),
    (36, 219, 0),
    (36, 219, 85),
    (36, 219, 170),
    (36, 219, 255),
    (36, 255, 0),
    (36, 255, 85),
    (36, 255, 170),
    (36, 255, 255),
    (73, 146, 170),
    (73, 146, 255),
    (73, 182, 0),
    (73, 182, 85),
    (73, 182, 170),
    (73, 182, 255),
    (73, 219, 0),
    (73, 219, 85),
    (73, 219, 170),
    (73, 219, 255),
    (73, 255, 0),
    (73, 255, 85),
    (73, 255, 170),
    (73, 255, 255),
    (109, 146, 85),
    (109, 146, 170),
    (109, 146, 255),
    (109, 182, 0),
    (109, 182, 85),
    (109, 182, 170),
    (109, 182, 255),
    (109, 219, 0),
    (109, 219, 85),
    (109, 219, 170),
    (109, 219, 255),
    (109, 255, 0),
    (109, 255, 85),
    (109, 255, 170),
    (109, 255, 255),
    (146, 146, 0),
    (146, 146, 85),
    (146, 146, 170),
    (146, 146, 255),
    (146, 182, 0),
    (146, 182, 85),
    (146, 182, 170),
    (146, 182, 255),
    (146, 219, 0),
    (146, 219, 85),
    (146, 219, 170),
    (146, 219, 255),
    (146, 255, 0),
    (146, 255, 85),
    (146, 255, 170),
    (146, 255, 255),
    (182, 109, 170),
    (182, 109, 255),
    (182, 146, 0),
    (182, 146, 85),
    (182, 146, 170),
    (182, 146, 255),
    (182, 182, 0),
    (182, 182, 85),
    (182, 182, 170),
    (182, 182, 255),
    (182, 219, 0),
    (182, 219, 85),
    (182, 219, 170),
    (182, 219, 255),
    (182, 255, 0),
    (182, 255, 85),
    (182, 255, 170),
    (182, 255, 255),
    (219, 109, 85),
    (219, 109, 170),
    (219, 109, 255),
    (219, 146, 0),
    (219, 146, 85),
    (219, 146, 170),
    (219, 146, 255),
    (219, 182, 0),
    (219, 182, 85),
    (219, 182, 170),
    (219, 182, 255),
    (219, 219, 0),
    (219, 219, 85),
    (219, 219, 170),
    (219, 219, 255),
    (219, 255, 0),
    (219, 255, 85),
    (219, 255, 170),
    (219, 255, 255),
    (255, 109, 0),
    (255, 109, 85),
    (255, 109, 170),
    (255, 109, 255),
    (255, 146, 0),
    (255, 146, 85),
    (255, 146, 170),
    (255, 146, 255),
    (255, 182, 0),
    (255, 182, 85),
    (255, 182, 170),
    (255, 182, 255),
    (255, 219, 0),
    (255, 219, 85),
    (255, 219, 170),
    (255, 219, 255),
    (255, 255, 0),
    (255, 255, 85),
    (255, 255, 170),
    (255, 255, 255),
];
