use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use combine::EasyParser;

use crate::{
    board_connection::WriteToBoard,
    kilter_data::{ClimbFilter, KilterData, placements_and_roles},
    placement_indicator::PlacementIndicator,
};

#[derive(Resource)]
pub struct SelectedClimb(pub String);

#[derive(Component)]
pub struct Board;

#[derive(Event)]
pub enum ChangeClimbEvent {
    Prev,
    Next,
    SelectByUuid(String),
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

#[derive(Resource)]
pub struct BoardAngle(pub u32);
impl Default for BoardAngle {
    fn default() -> Self {
        Self(40)
    }
}
impl BoardAngle {
    pub fn next(&self) -> Self {
        Self(if self.0 + BOARD_ANGLE_STEP > MAX_BOARD_ANGLE {
            MIN_BOARD_ANGLE
        } else {
            self.0 + BOARD_ANGLE_STEP
        })
    }
}

pub const BOARD_HEIGHT: f32 = 3.9;
pub const MIN_BOARD_ANGLE: u32 = 0;
pub const MAX_BOARD_ANGLE: u32 = 70;
pub const BOARD_ANGLE_STEP: u32 = 5;

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
        .init_resource::<BoardAngle>()
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
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -0.9, 0.3, 0.0),
            ..default()
        },
        // Tighten the shadow bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
    ));

    let board_width = 1477. / 1200. * BOARD_HEIGHT;

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(board_width, BOARD_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("original-16x12.png")),
            ..default()
        })),
        Board,
    ));

    // TODO: adjust scene so the floor is at y=0
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(3.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform {
            translation: Vec3::new(0., -BOARD_HEIGHT / 2., 0.),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            ..default()
        },
    ));
}

// TODO move to keyboard.rs or something
fn prev_next_climb(keys: Res<ButtonInput<KeyCode>>, mut writer: EventWriter<ChangeClimbEvent>) {
    if keys.just_pressed(KeyCode::ArrowRight) {
        writer.write(ChangeClimbEvent::Next);
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        writer.write(ChangeClimbEvent::Prev);
    }
}

fn change_climb(
    mut selected: ResMut<SelectedClimb>,
    filter: Res<ClimbFilter>,
    mut reader: EventReader<ChangeClimbEvent>,
) {
    for event in reader.read() {
        match event {
            ChangeClimbEvent::Prev => {
                if filter.filtered_climbs.is_empty() {
                    continue;
                }

                let current_index = filter
                    .filtered_climbs
                    .get_index_of(&selected.0)
                    .unwrap_or(0);

                let prev_index = if current_index == 0 {
                    filter.filtered_climbs.len() - 1
                } else {
                    current_index - 1
                };

                debug!("Navigating. {current_index} -> {prev_index}");

                if let Some(prev_uuid) = filter.filtered_climbs.get_index(prev_index) {
                    selected.0 = prev_uuid.clone();
                }
            }
            ChangeClimbEvent::Next => {
                if filter.filtered_climbs.is_empty() {
                    continue;
                }

                let current_index = filter
                    .filtered_climbs
                    .get_index_of(&selected.0)
                    .unwrap_or(0);

                let next_index = if current_index + 1 >= filter.filtered_climbs.len() {
                    0
                } else {
                    current_index + 1
                };

                debug!("Navigating. {current_index} -> {next_index}");

                if let Some(next_uuid) = filter.filtered_climbs.get_index(next_index) {
                    selected.0 = next_uuid.clone();
                }
            }
            ChangeClimbEvent::SelectByUuid(uuid) => selected.0 = uuid.clone(),
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
    mut events: EventWriter<WriteToBoard>,
) {
    if !selected.is_added() && !selected.is_changed() && !settings.is_changed() {
        return;
    }

    let Ok(board) = boards.single() else {
        return;
    };

    let Some(climb) = kilter.climbs.get(&selected.0) else {
        return;
    };

    for entity in &indicators {
        commands.entity(entity).despawn();
    }

    let Ok((placements, _)) = placements_and_roles().easy_parse(climb.frames.as_str()) else {
        return;
    };

    for (placement_id, role_id) in &placements {
        let indicator = commands
            .spawn(PlacementIndicator {
                placement_id: *placement_id,
                role_id: *role_id,
            })
            .id();

        commands.entity(board).add_child(indicator);
    }

    debug!("Showing frames: {}", climb.frames);

    events.write(WriteToBoard::from_positions_and_roles(&placements, &kilter));
}
