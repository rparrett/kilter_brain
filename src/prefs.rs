use std::path::PathBuf;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin as SimplePrefsPlugin, PrefsStatus};

use crate::kilter_data::{Climb, ClimbFilter, KilterData};

#[derive(Prefs, Reflect, Default)]
struct UserPrefs {
    climbs: UserClimbs,
}

#[derive(Resource, Reflect, Clone, Eq, PartialEq, Debug, Default)]
pub struct UserClimbs(pub HashMap<String, Climb>);

pub struct PrefsPlugin;

impl Plugin for PrefsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SimplePrefsPlugin::<UserPrefs> {
            #[cfg(not(target_arch = "wasm32"))]
            path: ensure_prefs_dir().expect("Failed to set up prefs storage"),
            ..default()
        });

        app.add_systems(Update, check_status);
    }
}

fn check_status(
    status: Res<PrefsStatus<UserPrefs>>,
    user_climbs: Res<UserClimbs>,
    mut kilter: ResMut<KilterData>,
    mut filter: ResMut<ClimbFilter>,
    mut inserted: Local<bool>,
) {
    if status.loaded && !*inserted {
        for (k, v) in &user_climbs.0 {
            kilter.climbs.insert(k.clone(), v.clone());
        }
        filter.update(&kilter);
        *inserted = true;
    }
}

fn ensure_prefs_dir() -> Result<PathBuf> {
    let dir = dirs::config_local_dir()
        .ok_or("Failed to determine local config directory")?
        .join("kilter_brain");

    std::fs::create_dir_all(&dir)?;

    Ok(dir.join("prefs.ron"))
}
