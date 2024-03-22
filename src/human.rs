use std::f32::consts::PI;

use bevy::prelude::*;

use crate::BOARD_HEIGHT;

pub struct HumanPlugin;

impl Plugin for HumanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HumanAssets>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, setup_scene_once_loaded);
    }
}

#[derive(Resource)]
struct HumanAssets {
    scene: Handle<Scene>,
    animations: Vec<Handle<AnimationClip>>,
}
impl FromWorld for HumanAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            scene: asset_server.load("human.glb#Scene0"),
            animations: vec![asset_server.load("human.glb#Animation1")],
        }
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    assets: Res<HumanAssets>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(assets.animations[0].clone_weak()).repeat();
    }
}

fn setup(mut commands: Commands, assets: Res<HumanAssets>) {
    commands.spawn(SceneBundle {
        scene: assets.scene.clone(),
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::XZY, -PI / 2., 3.6, 0.0),
            scale: Vec3::splat(1.76 / 2.),
            translation: Vec3::new(1.9, -BOARD_HEIGHT / 2., 1.6),
        },
        ..default()
    });
}
