use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use combine::EasyParser;

use crate::{
    kilter_data::{placements_and_roles, KilterData},
    placement_indicator::PlacementIndicator,
};

#[derive(Resource, Default)]
pub struct SelectedClimb(pub usize);

#[derive(Component)]
pub struct Board;

#[derive(Event)]
pub enum ChangeClimbEvent {
    Prev,
    Next,
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct KilterSettings {
    pub offset: Vec2,
    pub scale: f32,
}
impl Default for KilterSettings {
    fn default() -> Self {
        Self {
            offset: Vec2::new(-1.81, -1.96),
            scale: 0.0251,
        }
    }
}

pub const BOARD_HEIGHT: f32 = 3.9;

pub struct KilterBoardPlugin;

impl Plugin for KilterBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                show_climb.before(crate::placement_indicator::update),
                prev_next_climb,
                change_climb,
            ),
        )
        .add_systems(Startup, setup_scene)
        .add_event::<ChangeClimbEvent>()
        .init_resource::<SelectedClimb>()
        .init_resource::<KilterSettings>()
        .register_type::<KilterSettings>();
    }
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -0.9, 0.3, 0.0),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });

    let board_width = 1477. / 1200. * BOARD_HEIGHT;

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Rectangle::new(board_width, BOARD_HEIGHT)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("original-16x12.png")),
                ..default()
            }),
            ..default()
        },
        Board,
    ));

    // TODO: adjust scene so the floor is at y=0
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(3.0)),
        material: materials.add(Color::WHITE),
        transform: Transform {
            translation: Vec3::new(0., -BOARD_HEIGHT / 2., 0.),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ..default()
        },
        ..default()
    });
}

// TODO move to keyboard.rs or something
fn prev_next_climb(keys: Res<ButtonInput<KeyCode>>, mut writer: EventWriter<ChangeClimbEvent>) {
    if keys.just_pressed(KeyCode::ArrowRight) {
        writer.send(ChangeClimbEvent::Next);
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        writer.send(ChangeClimbEvent::Prev);
    }
}

fn change_climb(
    mut selected: ResMut<SelectedClimb>,
    kilter: Res<KilterData>,
    mut reader: EventReader<ChangeClimbEvent>,
) {
    for event in reader.read() {
        match event {
            ChangeClimbEvent::Prev => {
                selected.0 = if selected.0 == 0 {
                    kilter.climbs.len() - 1
                } else {
                    selected.0 - 1
                };
            }
            ChangeClimbEvent::Next => {
                selected.0 = if selected.0 + 1 >= kilter.climbs.len() {
                    0
                } else {
                    selected.0 + 1
                };
            }
        }
    }
}

fn show_climb(
    mut commands: Commands,
    selected: Res<SelectedClimb>,
    kilter: Res<KilterData>,
    settings: Res<KilterSettings>,
    indicators: Query<Entity, With<PlacementIndicator>>,
    boards: Query<Entity, With<Board>>,
) {
    if !selected.is_added() && !selected.is_changed() && !settings.is_changed() {
        return;
    }

    let board = boards.single();

    // Get selected or first climb
    let Some(climb) = kilter
        .climbs
        .iter()
        .nth(selected.0)
        .or_else(|| kilter.climbs.iter().next())
        .map(|(_, climb)| climb)
    else {
        return;
    };

    for entity in &indicators {
        commands.entity(entity).despawn_recursive();
    }

    let Ok((placements, _)) = placements_and_roles().easy_parse(climb.frames.as_str()) else {
        return;
    };

    for (placement_id, role_id) in placements {
        let indicator = commands
            .spawn(PlacementIndicator {
                placement_id,
                role_id,
            })
            .id();

        commands.entity(board).add_child(indicator);
    }
}
