use bevy::{
    input::gestures::PinchGesture,
    picking::events::{Click, DragEnd, Pointer},
    prelude::*,
};

use uuid::Uuid;

use std::fmt::Write;

use crate::{
    clipboard::PasteEvent,
    kilter_board::{Board, KilterSettings, SelectedClimb},
    kilter_data::{parse_placements_and_roles, Climb, KilterData},
    placement_indicator::PlacementIndicator,
};

pub struct AuthoringPlugin;

impl Plugin for AuthoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (cycle, log_frames, on_paste));
    }
}

fn cycle(
    mut commands: Commands,
    mut indicator_query: Query<(Entity, &mut PlacementIndicator)>,
    board_query: Query<(Entity, &GlobalTransform), With<Board>>,
    mut click_events: EventReader<Pointer<Click>>,
    mut drag_end: EventReader<Pointer<DragEnd>>,
    mut pinch_events: EventReader<PinchGesture>,
    kilter: Res<KilterData>,
    settings: Res<KilterSettings>,
) {
    let pinching = pinch_events.read().len() > 0;
    let drag_dist = drag_end.read().map(|e| e.event.distance).sum::<Vec2>();

    for event in click_events.read() {
        let Ok((board_entity, board)) = board_query.get(event.target) else {
            continue;
        };

        if drag_dist.length_squared() > 256.0 {
            continue;
        }

        if pinching {
            continue;
        }

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

            if min.map_or(true, |(_, _, min_d_squared)| d_squared < min_d_squared) {
                min = Some((*id, pos, d_squared));
            }
        }

        let Some((placement_id, _, _d_squared)) = min else {
            continue;
        };

        // TODO consider monitoring the selected climb's frame data directly
        // and updating the indicators in a separate system.

        let search = indicator_query
            .iter_mut()
            .find(|(_, p)| p.placement_id == placement_id);

        // Determine the order of roles to cycle through.

        let first_role_id = kilter
            .placements
            .get(&placement_id)
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
            None,
        ];

        if let Some(first_pos) = roles.iter().position(|r| *r == Some(first_role_id)) {
            if first_pos != 0 {
                roles.swap(first_pos, 0);
            }
        }

        if let Some((entity, mut placement)) = search {
            let current = roles
                .iter()
                .position(|r| *r == Some(placement.role_id))
                .unwrap();
            let next = roles.iter().cycle().nth(current + 1).unwrap();

            if let Some(next) = next {
                placement.role_id = *next;
            } else {
                commands.entity(entity).despawn_recursive();
            }
        } else {
            // TODO if there are already two start holds on the board,
            // don't use that role even if it's the default.

            let indicator = commands
                .spawn(PlacementIndicator {
                    placement_id,
                    role_id: roles.first().unwrap().unwrap(),
                })
                .id();
            commands.entity(board_entity).add_child(indicator);
        }
    }
}

fn log_frames(
    query: Query<&PlacementIndicator>,
    changed_query: Query<(), Changed<PlacementIndicator>>,
) {
    // TODO this iterates the entire query. Use an event or something.
    if changed_query.is_empty() {
        return;
    }

    let out: String = query.iter().fold(String::new(), |mut out, ind| {
        let _ = write!(out, "{ind}");
        out
    });

    info!("{out}");
}

fn on_paste(
    mut events: EventReader<PasteEvent>,
    mut selected: ResMut<SelectedClimb>,
    mut kilter: ResMut<KilterData>,
) {
    for event in events.read() {
        let mut added = 0;

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

            kilter.climbs.insert(
                id.clone(),
                Climb {
                    uuid: id.clone(),
                    setter_username: "User".to_string(),
                    name: name.to_string(),
                    frames: frames.to_string(),
                    ..default()
                },
            );

            added += 1;
        }

        if added > 0 {
            selected.0 = kilter.climbs.len() - added;
        }
    }
}
