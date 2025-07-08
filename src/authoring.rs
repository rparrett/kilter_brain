use bevy::{
    input::gestures::PinchGesture,
    picking::events::{Click, DragEnd, Pointer},
    platform::collections::HashMap,
    prelude::*,
};

use combine::EasyParser;
use uuid::Uuid;

use crate::{
    clipboard::PasteEvent,
    kilter_board::{Board, ChangeClimbEvent, KilterSettings, SelectedClimb},
    kilter_data::{
        Climb, ClimbFilter, KilterData, parse_placements_and_roles, placements_and_roles,
    },
};

pub struct AuthoringPlugin;

impl Plugin for AuthoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (cycle, on_paste));
    }
}

fn cycle(
    board_query: Query<&GlobalTransform, With<Board>>,
    mut click_events: EventReader<Pointer<Click>>,
    mut drag_end: EventReader<Pointer<DragEnd>>,
    mut pinch_events: EventReader<PinchGesture>,
    mut kilter: ResMut<KilterData>,
    settings: Res<KilterSettings>,
    selected: Res<SelectedClimb>,
    mut events: EventWriter<ChangeClimbEvent>,
) {
    let pinching = pinch_events.read().len() > 0;
    let drag_dist = drag_end.read().map(|e| e.event.distance).sum::<Vec2>();

    for event in click_events.read() {
        let Ok(board) = board_query.get(event.target) else {
            continue;
        };

        if drag_dist.length_squared() > 256.0 {
            continue;
        }

        if pinching {
            continue;
        }

        let Some(climb) = kilter.climbs.get(&selected.0) else {
            continue;
        };

        let placements = placements_and_roles()
            .easy_parse(climb.frames.as_str())
            .map(|(pr, _err)| pr)
            .unwrap_or_else(|_| Vec::new());

        let mut placements = HashMap::<u32, u32>::from_iter(placements);

        // Find closest placement to cursor hit position

        let mut min: Option<(u32, Vec2, f32)> = None;

        for (id, placement) in &kilter.placements {
            if placement.layout_id != 1 {
                continue;
            }

            let Some(hole) = kilter.holes.get(&placement.hole_id) else {
                continue;
            };

            let Some(hit_position) = event.event.hit.position else {
                continue;
            };

            let pos = Vec2::new(hole.x as f32, hole.y as f32) * settings.scale + settings.offset;

            let cursor = hit_position - board.translation();

            let d_squared = pos.distance_squared(cursor.truncate());

            if min.is_none_or(|(_, _, min_d_squared)| d_squared < min_d_squared) {
                min = Some((*id, pos, d_squared));
            }
        }

        let Some((closest_placement_id, _, _d_squared)) = min else {
            continue;
        };

        // Determine the order of roles to cycle through.

        let first_role_id = kilter
            .placements
            .get(&closest_placement_id)
            .and_then(|p| p.default_placement_role_id)
            .unwrap_or(13);

        let mut roles = [
            // any
            Some(13),
            // foot only
            Some(15),
            // start
            Some(12),
            // finish
            Some(14),
            // CUSTOM cheat hold
            Some(99),
            // CUSTOM match allowed
            Some(98),
            None,
        ];

        if let Some(first_pos) = roles.iter().position(|r| *r == Some(first_role_id)) {
            if first_pos != 0 {
                roles.swap(first_pos, 0);
            }
        }

        // Update placement in hashmap

        let mut new_placement = false;
        let entry = placements.entry(closest_placement_id).or_insert_with(|| {
            new_placement = true;
            roles.first().unwrap().unwrap()
        });
        let current = *entry;
        let current_ind = roles.iter().position(|r| *r == Some(current)).unwrap();

        if !new_placement {
            match roles.iter().cycle().nth(current_ind + 1).unwrap() {
                Some(next) => {
                    info!("next: {next}");
                    *entry = *next
                }
                None => {
                    placements.remove(&closest_placement_id);
                }
            }
        }

        // Update frames

        let frames = placements.iter().fold(String::new(), |mut acc, (k, v)| {
            acc.push_str(&format!("p{k}r{v}"));
            acc
        });

        let Some(climb) = kilter.climbs.get_mut(&selected.0) else {
            continue;
        };

        climb.frames = frames;

        // Cause board to be updated, and new frames to be sent over bluetooth
        events.write(ChangeClimbEvent::SelectByUuid(climb.uuid.clone()));
    }
}

fn on_paste(
    mut events: EventReader<PasteEvent>,
    mut selected: ResMut<SelectedClimb>,
    mut kilter: ResMut<KilterData>,
    mut filter: ResMut<ClimbFilter>,
) {
    for event in events.read() {
        let mut to_add = vec![];

        let lines = event.0.split('\n');
        for (l, line) in lines.enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Accept `name,frames` or `frames`.
            let mut parts = line.rsplit(',');
            let Some(frames) = parts.next() else {
                continue;
            };
            let name = parts.next().unwrap_or("Pasted Climb");

            if let Err(e) = parse_placements_and_roles(frames) {
                // TODO add UI toast thing to show errors
                warn!("On pasted line {}: {}", l, e);
                continue;
            }

            let id = Uuid::new_v4().simple().to_string();

            to_add.push(Climb {
                uuid: id.clone(),
                setter_username: "User".to_string(),
                name: name.to_string(),
                frames: frames.to_string(),
                ..default()
            });

            selected.0 = id.clone();
        }

        if !to_add.is_empty() {
            filter.override_climbs.clear();
            for climb in to_add {
                filter.override_climbs.insert(climb.uuid.clone());
                kilter.climbs.insert(climb.uuid.clone(), climb);
            }
            filter.update(&kilter);
        }
    }
}
