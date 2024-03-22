use bevy::prelude::*;

use crate::{kilter_data::KilterData, Board, KilterSettings};

pub struct AuthoringPlugin;

impl Plugin for AuthoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_cursor);
    }
}

const PICKING_THRESHOLD: f32 = 0.01; // squared distance

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    board_query: Query<&GlobalTransform, With<Board>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
    settings: Res<KilterSettings>,
    kilter: Res<KilterData>,
) {
    let (camera, camera_transform) = camera_query.single();
    let board = board_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the board plane.
    let Some(distance) = ray.intersect_plane(board.translation(), Plane3d::new(board.forward()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Find the closest placement

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

    let Some((_placement_id, pos, d_squared)) = min else {
        return;
    };

    if d_squared > PICKING_THRESHOLD {
        return;
    }

    gizmos.circle(
        board.translation() + pos.extend(0.) - board.forward() * 0.01,
        Direction3d::new_unchecked(board.forward()),
        0.1,
        Color::WHITE,
    );
}
