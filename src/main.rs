use authoring::AuthoringPlugin;
use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, utils::Uuid};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use button::ButtonPlugin;
use combine::EasyParser;

use clipboard::ClipboardPlugin;
use human::HumanPlugin;
use kilter_data::{placements_and_roles, Climb, KilterData};
use panels::PanelsPlugin;
use placement_indicator::{PlacementIndicator, PlacementIndicatorPlugin};

mod authoring;
mod button;
#[cfg_attr(not(target_arch = "wasm32"), path = "native_clipboard.rs")]
#[cfg_attr(target_arch = "wasm32", path = "wasm_clipboard.rs")]
mod clipboard;
mod human;
mod kilter_data;
mod palette;
mod panels;
mod placement_indicator;
mod theme;

#[derive(Event)]
struct PasteEvent(String);

#[derive(Resource, Default)]
struct SelectedClimb(String);

#[derive(Component)]
struct Board;

#[derive(Reflect, Resource)]
#[reflect(Resource)]
struct KilterSettings {
    offset: Vec2,
    scale: f32,
}
impl Default for KilterSettings {
    fn default() -> Self {
        Self {
            offset: Vec2::new(-1.81, -1.96),
            scale: 0.0251,
        }
    }
}

const BOARD_HEIGHT: f32 = 3.9;

fn main() {
    // Just embed some minimal json on the web for now. In the future we will want to
    // be able to load this data from an API endpoint or perhaps just through Bevy's
    // asset server.
    #[cfg(target_arch = "wasm32")]
    let kd = {
        let mut kd = KilterData::default();
        kd.json_update_reader(std::io::Cursor::new(include_str!("../minimal.json")));
        kd
    };
    #[cfg(not(target_arch = "wasm32"))]
    let kd = {
        let mut kd = KilterData::from_sqlite("../kilter_brain_data/db.sqlite3").unwrap();
        if let Err(e) = kd.json_update_files("../kilter_brain_data/api_json") {
            eprintln!("Failed to load JSON updates. {:?}", e);
        };
        kd
    };

    App::new()
        .insert_resource(kd)
        .add_event::<PasteEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            ClipboardPlugin,
            HumanPlugin,
            AuthoringPlugin,
            ButtonPlugin,
            PanelsPlugin,
            PlacementIndicatorPlugin,
        ))
        .add_plugins((
            ResourceInspectorPlugin::<KilterSettings>::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (show_climb, next_climb, on_paste))
        .init_resource::<SelectedClimb>()
        .init_resource::<KilterSettings>()
        .register_type::<KilterSettings>()
        .run();
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

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 1.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
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

fn next_climb(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedClimb>,
    kilter: Res<KilterData>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let Some(mut next) = kilter.climbs.iter().next().map(|(id, _climb)| id) else {
            return;
        };

        let mut iter = kilter.climbs.iter();

        for (id, _climb) in iter.by_ref() {
            if *id == selected.0 {
                break;
            }
        }

        if let Some((id, _climb)) = iter.next() {
            next = id;
        }

        selected.0 = next.clone();
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

    let Some(climb) = kilter
        .climbs
        .get(&selected.0)
        .or_else(|| kilter.climbs.iter().next().map(|(_id, climb)| climb))
    else {
        return;
    };

    let Ok((placements, _)) = placements_and_roles().easy_parse(climb.frames.as_str()) else {
        return;
    };

    for entity in &indicators {
        commands.entity(entity).despawn_recursive();
    }

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

fn on_paste(
    mut events: EventReader<PasteEvent>,
    mut selected: ResMut<SelectedClimb>,
    mut kilter: ResMut<KilterData>,
) {
    for event in events.read() {
        let id = Uuid::new_v4().simple().to_string();

        // TODO handle pastes with multiple climbs, either in the format
        //
        // p#r#p#r#\np#r#p#r#\np#r#p#r#
        // or
        // name\np#r#p#r#\nname\np#r#p#r#
        //
        // blocking: the p#r# parser needs to require at least one p#r#.

        // Handle frame data, or "name\nframe_data"
        let mut parts = event.0.trim().rsplit('\n');
        let frames = parts.next().unwrap();
        let name = parts.next().unwrap_or("Pasted Climb");

        match placements_and_roles().easy_parse(frames) {
            Ok(_) => {
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
                selected.0 = id;
            }
            Err(e) => {
                warn!("{:?}", e);
                return;
            }
        }
    }
}
