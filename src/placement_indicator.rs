use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use bevy_mod_picking::picking_core::Pickable;

use std::fmt::Display;

use crate::{kilter_data::KilterData, KilterSettings};

pub struct PlacementIndicatorPlugin;

impl Plugin for PlacementIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IndicatorHandles>();
        app.add_systems(Update, update);
    }
}

#[derive(Resource)]
struct IndicatorHandles {
    materials: HashMap<String, Handle<StandardMaterial>>,
    mesh: Handle<Mesh>,
    outline_mesh: Handle<Mesh>,
}
impl FromWorld for IndicatorHandles {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();

        Self {
            mesh: meshes.add(Circle::new(0.03)),
            outline_mesh: meshes.add(Circle::new(0.04)),
            materials: HashMap::default(),
        }
    }
}
#[derive(SystemParam)]
pub struct IndicatorHandlesParam<'w> {
    handles: ResMut<'w, IndicatorHandles>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
}
impl IndicatorHandlesParam<'_> {
    fn get_material(&mut self, color: &str) -> Handle<StandardMaterial> {
        if let Some(mat) = self.handles.materials.get(color) {
            return mat.clone();
        };

        let material = StandardMaterial {
            base_color: Srgba::hex(color).unwrap().into(),
            unlit: true,
            ..default()
        };

        self.materials.add(material)
    }
}

#[derive(Component)]
pub struct PlacementIndicator {
    pub placement_id: u32,
    pub role_id: u32,
}
impl Display for PlacementIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}r{}", self.placement_id, self.role_id)
    }
}

pub fn update(
    mut commands: Commands,
    mut query: Query<(Entity, Ref<PlacementIndicator>), Changed<PlacementIndicator>>,
    kilter: Res<KilterData>,
    settings: Res<KilterSettings>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    mut handles: IndicatorHandlesParam,
) {
    for (entity, indicator) in &mut query {
        let Some(placement) = kilter.placements.get(&indicator.placement_id) else {
            warn!("missing placement: {}", indicator.placement_id);
            continue;
        };
        let Some(role) = kilter.placement_roles.get(&indicator.role_id) else {
            warn!("missing role: {}", indicator.role_id);
            continue;
        };
        let Some(hole) = kilter.holes.get(&placement.hole_id) else {
            warn!("missing hole: {}", placement.hole_id);
            continue;
        };

        if indicator.is_added() {
            let pos = Vec2::new(hole.x as f32, hole.y as f32) * settings.scale + settings.offset;

            // Outline
            let outline = commands
                .spawn((
                    PbrBundle {
                        mesh: handles.handles.outline_mesh.clone(),
                        material: handles.get_material("#000000"),
                        transform: Transform::from_translation(Vec3::Z * -0.0001),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id();

            commands.entity(entity).insert((
                PbrBundle {
                    mesh: handles.handles.mesh.clone(),
                    material: handles.get_material(&role.led_color),
                    transform: Transform::from_translation(pos.extend(0.0002)),
                    ..default()
                },
                Pickable::IGNORE,
            ));

            commands.entity(entity).add_child(outline);
        } else {
            let Ok(mut mat) = material_query.get_mut(entity) else {
                continue;
            };

            *mat = handles.get_material(&role.led_color);
        }
    }
}
