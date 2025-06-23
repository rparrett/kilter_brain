use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;

use crate::kilter_board::BOARD_HEIGHT;

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
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}
impl FromWorld for HumanAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("human.glb"));

        let mut graph = AnimationGraph::new();
        let animations = graph
            .add_clips(
                [GltfAssetLabel::Animation(1).from_asset("human.glb")]
                    .into_iter()
                    .map(|path| asset_server.load(path)),
                1.0,
                graph.root,
            )
            .collect();

        let mut graphs = world.resource_mut::<Assets<AnimationGraph>>();
        let graph = graphs.add(graph);

        Self {
            scene,
            animations,
            graph,
        }
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    assets: Res<HumanAssets>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, assets.animations[0], Duration::ZERO)
            .repeat();

        player.play(assets.animations[0]).repeat();

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(assets.graph.clone()))
            .insert(transitions);
    }
}

fn setup(mut commands: Commands, assets: Res<HumanAssets>) {
    commands.spawn((
        SceneRoot(assets.scene.clone()),
        Transform {
            rotation: Quat::from_euler(EulerRot::XZY, -PI / 2., 3.6, 0.0),
            scale: Vec3::splat(1.76 / 2.),
            translation: Vec3::new(1.9, -BOARD_HEIGHT / 2., 1.6),
        },
    ));
}
