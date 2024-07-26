use bevy::{
    input::{
        gestures::PinchGesture,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    prelude::*,
};
use bevy_mod_picking::events::{Drag, Pointer};

#[derive(Default)]
pub struct PanCamPlugin;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_movement, camera_zoom))
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    // camera
    let pos = Vec3::new(-2.0, 1.0, 6.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(pos).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanCam {
            bounds_min: pos.truncate(),
            bounds_max: pos.truncate(),
            ..default()
        },
    ));
}

fn camera_zoom(
    mut query: Query<(&mut PanCam, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    mut pinch_events: EventReader<PinchGesture>,
) {
    let pixels_per_line = 100.;
    let mut scroll = scroll_events
        .read()
        .map(|ev| match ev.unit {
            MouseScrollUnit::Pixel => ev.y,
            MouseScrollUnit::Line => ev.y * pixels_per_line,
        })
        .sum::<f32>();

    scroll += pinch_events.read().map(|gesture| gesture.0).sum::<f32>() * -100.;

    if scroll == 0. {
        return;
    }

    for (mut cam, mut pos) in &mut query {
        let anim_start_pos = Vec3::new(-2.0, 1.0, 6.0);
        let anim_start_transform =
            Transform::from_translation(anim_start_pos).looking_at(Vec3::ZERO, Vec3::Y);
        let anim_start_rotation = anim_start_transform.rotation;

        let anim_end_pos = Vec3::new(0.0, 0.0, 6.0);
        let anim_end_rotation = Quat::IDENTITY;

        let start_bounds_min = Vec2::new(0., 0.);
        let start_bounds_max = Vec2::new(0., 0.);
        let end_bounds_min = Vec2::new(-2., -2.);
        let end_bounds_max = Vec2::new(2., 2.);

        let max_z = anim_end_pos.z;
        let min_z = 1.0;

        cam.current_zoom = (cam.current_zoom - scroll / 500.).clamp(0., 1.0);

        let actual_zoom = ((cam.current_zoom - 0.2) / 0.8).clamp(0.0, 1.0);
        let anim_progress = (cam.current_zoom / 0.2).clamp(0.0, 1.0);

        let (bounds_min, bounds_max) = if anim_progress < 1. {
            (
                anim_start_pos.lerp(anim_end_pos, anim_progress).truncate(),
                anim_start_pos.lerp(anim_end_pos, anim_progress).truncate(),
            )
        } else {
            (
                start_bounds_min.lerp(end_bounds_min, actual_zoom),
                start_bounds_max.lerp(end_bounds_max, actual_zoom),
            )
        };

        cam.bounds_min = bounds_min;
        cam.bounds_max = bounds_max;

        let rot = anim_start_rotation.slerp(anim_end_rotation, anim_progress);

        pos.translation.x = pos.translation.x.max(bounds_min.x).min(bounds_max.x);
        pos.translation.y = pos.translation.y.max(bounds_min.y).min(bounds_max.y);
        pos.translation.z = max_z.lerp(min_z, actual_zoom);
        pos.rotation = rot;
    }
}

fn camera_movement(
    mut query: Query<(&mut PanCam, &mut Transform)>,
    mut drag_events: EventReader<Pointer<Drag>>,
) {
    for e in drag_events.read() {
        for (cam, mut transform) in &mut query {
            // TODO observed board movement should be 1-1 with cursor movement
            let delta = e.event.delta * Vec2::new(1., -1.) / cam.bounds_max.x / 40.;
            let proposed_cam_transform = transform.translation - delta.extend(0.);

            transform.translation = proposed_cam_transform;
            transform.translation.x = transform
                .translation
                .x
                .max(cam.bounds_min.x)
                .min(cam.bounds_max.x);
            transform.translation.y = transform
                .translation
                .y
                .max(cam.bounds_min.y)
                .min(cam.bounds_max.y);
        }
    }
}

#[derive(Default, Component)]
struct PanCam {
    current_zoom: f32,
    bounds_min: Vec2,
    bounds_max: Vec2,
}
