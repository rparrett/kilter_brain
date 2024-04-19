use bevy::prelude::*;
use bevy_mod_picking::events::{Click, Move, Out, Pointer};

use std::fmt::Write;

use crate::{
    kilter_data::KilterData, placement_indicator::PlacementIndicator, Board, KilterSettings,
};

pub struct AuthoringPlugin;

impl Plugin for AuthoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedPlacement>();
        app.add_systems(Update, (cycle, log_frames, picking, draw_selection));
    }
}

#[derive(Resource, Default)]
struct SelectedPlacement(Option<u32>);

const PICKING_THRESHOLD: f32 = 0.01; // squared distance

fn picking(
    mut move_events: EventReader<Pointer<Move>>,
    mut out_events: EventReader<Pointer<Out>>,
    board_query: Query<&GlobalTransform, With<Board>>,
    kilter: Res<KilterData>,
    settings: Res<KilterSettings>,
    mut selected: ResMut<SelectedPlacement>,
) {
    for event in move_events.read() {
        info!("{:?}", event);

        let Ok(board) = board_query.get(event.target) else {
            continue;
        };

        let Some(point) = event.event.hit.position else {
            continue;
        };

        let mut min: Option<(u32, Vec2, f32)> = None;

        for (id, placement) in &kilter.placements {
            if placement.layout_id != 1 {
                continue;
            }

            let Some(hole) = kilter.holes.get(&placement.hole_id) else {
                continue;
            };

            let pos = Vec2::new(hole.x as f32, hole.y as f32) * settings.scale + settings.offset;

            let cursor = point - board.translation();

            let d_squared = pos.distance_squared(cursor.truncate());

            if min.map_or(true, |(_, _, min_d_squared)| d_squared < min_d_squared) {
                min = Some((*id, pos, d_squared));
            }
        }

        let Some((placement_id, _, d_squared)) = min else {
            selected.0 = None;
            return;
        };

        if d_squared > PICKING_THRESHOLD {
            selected.0 = None;
            return;
        }

        selected.0 = Some(placement_id);
    }

    for _ in out_events.read() {
        selected.0 = None;
    }
}

fn draw_selection(
    mut gizmos: Gizmos,
    kilter: Res<KilterData>,
    settings: Res<KilterSettings>,
    board_query: Query<&GlobalTransform, With<Board>>,
    selected: Res<SelectedPlacement>,
) {
    let Ok(board) = board_query.get_single() else {
        return;
    };

    let Some(placement_id) = selected.0 else {
        return;
    };

    let Some(placement) = kilter.placements.get(&placement_id) else {
        return;
    };

    let Some(hole) = kilter.holes.get(&placement.hole_id) else {
        return;
    };

    let pos = Vec2::new(hole.x as f32, hole.y as f32) * settings.scale + settings.offset;

    gizmos.circle(
        board.translation() + pos.extend(0.) - board.forward() * 0.01,
        Direction3d::new_unchecked(board.forward()),
        0.1,
        Color::WHITE,
    );
}

fn cycle(
    mut commands: Commands,
    selected: Res<SelectedPlacement>,
    mut indicator_query: Query<(Entity, &mut PlacementIndicator)>,
    board_query: Query<Entity, With<Board>>,
    mut click_events: EventReader<Pointer<Click>>,
) {
    let Some(selected) = selected.0 else {
        return;
    };

    for event in click_events.read() {
        let Ok(_) = board_query.get(event.target) else {
            continue;
        };

        // TODO consider monitoring the selected climb's frame data directly
        // and updating the indicators in a separate system.

        let search = indicator_query
            .iter_mut()
            .find(|(_, p)| p.placement_id == selected);

        if let Some((entity, mut placement)) = search {
            // 12=start, 13=any, 15=foot_only, 14=finish

            let next = match placement.role_id {
                13 => Some(15),
                15 => Some(14),
                14 => None,
                _ => Some(13),
            };

            if let Some(next) = next {
                placement.role_id = next;
            } else {
                commands.entity(entity).despawn_recursive();
            }
        } else {
            // TODO use the default role for the placement as defined
            // in the database.

            let entity = board_query.single();
            let indicator = commands
                .spawn(PlacementIndicator {
                    placement_id: selected,
                    role_id: 12,
                })
                .id();
            commands.entity(entity).add_child(indicator);
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
