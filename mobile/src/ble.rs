use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ffi;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleState {
    pub is_on: bool,
    pub is_scanning: bool,
    pub devices: Vec<BleDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleDevice {
    pub id: String,
    pub name: String,
    pub advertised_name: String,
}

unsafe extern "C" {
    fn ble_start_scan();
    fn ble_stop_scan();
    fn ble_get_state_json() -> *const ffi::c_char;
    fn ble_free_string(ptr: *const ffi::c_char);
}

fn get_ble_state() -> Option<BleState> {
    unsafe {
        let ptr = ble_get_state_json();
        if ptr.is_null() {
            info!("BLE: rust ble state null pointer");
            return None;
        }

        // Convert C string to Rust String
        let c_str = ffi::CStr::from_ptr(ptr);
        let json_str = c_str.to_string_lossy();

        // Parse JSON into BleState struct
        info!("BLE: rust raw json: {}", json_str);
        let result = serde_json::from_str::<BleState>(&json_str).ok();

        ble_free_string(ptr); // Prevent memory leak
        result
    }
}

pub struct BlePlugin;
impl Plugin for BlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update);
        app.insert_resource(Time::<Fixed>::from_seconds(1.0));
    }
}

fn update() {
    let Some(state) = get_ble_state() else {
        info!("BLE: rust no state");
        return;
    };

    if !state.is_on {
        info!("BLE: rust ble off");
        return;
    }

    if !state.is_scanning {
        info!("BLE: rust starting scan");
        unsafe { ble_start_scan() };
        return;
    }

    info!("BLE: {:?}", state);
}
