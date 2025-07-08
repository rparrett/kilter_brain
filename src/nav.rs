use bevy::prelude::*;

use crate::{
    kilter_board::{BoardAngle, SelectedClimb},
    kilter_data::{ClimbFilter, KilterData},
};

pub struct NavPlugin;
impl Plugin for NavPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, data: Res<KilterData>, angle: Res<BoardAngle>) {
    let filter = ClimbFilter::new(angle.0, &data);
    commands.insert_resource(SelectedClimb(
        filter.filtered_climbs.get_index(0).unwrap().clone(),
    ));
    commands.insert_resource(filter);
}
