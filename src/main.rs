use authoring::AuthoringPlugin;
use bevy::{
    input::common_conditions::input_toggle_active, pbr::CascadeShadowConfigBuilder, prelude::*,
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_mod_picking::prelude::*;
use gen_api::GenApiPlugin;

use button::ButtonPlugin;
use combine::EasyParser;

use clipboard::ClipboardPlugin;
use human::HumanPlugin;
use kilter_data::{parse_placements_and_roles, placements_and_roles, Climb, KilterData};
use pan_cam::PanCamPlugin;
use panels::PanelsPlugin;
use placement_indicator::{PlacementIndicator, PlacementIndicatorPlugin};
use uuid::Uuid;

mod authoring;
mod button;
#[cfg_attr(not(target_arch = "wasm32"), path = "native_clipboard.rs")]
#[cfg_attr(target_arch = "wasm32", path = "wasm_clipboard.rs")]
mod clipboard;
mod gen_api;
mod human;
mod kilter_data;
mod pan_cam;
mod panels;
mod placement_indicator;
mod theme;

#[derive(Event)]
struct PasteEvent(String);

#[derive(Resource, Default)]
struct SelectedClimb(usize);

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
            GenApiPlugin,
            ClipboardPlugin,
            HumanPlugin,
            AuthoringPlugin,
            ButtonPlugin,
            PanelsPlugin,
            PlacementIndicatorPlugin,
            PanCamPlugin,
        ))
        .add_plugins((
            ResourceInspectorPlugin::<KilterSettings>::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
            DefaultPickingPlugins,
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                show_climb.before(placement_indicator::update),
                prev_next_climb,
                on_paste,
            ),
        )
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

fn prev_next_climb(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedClimb>,
    kilter: Res<KilterData>,
) {
    if keys.just_pressed(KeyCode::ArrowRight) {
        selected.0 = if selected.0 + 1 >= kilter.climbs.len() {
            0
        } else {
            selected.0 + 1
        };
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        selected.0 = if selected.0 == 0 {
            kilter.climbs.len() - 1
        } else {
            selected.0 - 1
        };
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
