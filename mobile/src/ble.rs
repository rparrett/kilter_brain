use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ffi;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleState {
    pub is_on: bool,
    pub is_scanning: bool,
    pub is_connected: bool,
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
    fn ble_connect(id_str: *const ffi::c_char) -> bool;
    fn ble_write_characteristic(
        service_uuid: *const ffi::c_char,
        characteristic_uuid: *const ffi::c_char,
        data: *const u8,
        data_length: usize,
    ) -> bool;
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

fn connect_to_device(device_id: &str) -> bool {
    unsafe {
        let c_str = ffi::CString::new(device_id).unwrap();
        ble_connect(c_str.as_ptr())
    }
}

pub fn write_to_characteristic(service_uuid: &str, characteristic_uuid: &str, data: &[u8]) -> bool {
    unsafe {
        let service_c_str = ffi::CString::new(service_uuid).unwrap();
        let characteristic_c_str = ffi::CString::new(characteristic_uuid).unwrap();

        ble_write_characteristic(
            service_c_str.as_ptr(),
            characteristic_c_str.as_ptr(),
            data.as_ptr(),
            data.len(),
        )
    }
}

pub struct BlePlugin;
impl Plugin for BlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update);
        app.insert_resource(Time::<Fixed>::from_seconds(1.0));
    }
}

fn update(mut connection_initialized: Local<bool>, mut wrote_test_data: Local<bool>) {
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

    for device in state.devices {
        if device.advertised_name.starts_with("Fake Kilter")
            && !*connection_initialized
            && !state.is_connected
        {
            info!("BLE: rust wants to connect to Fake Kilter");
            connect_to_device(&device.id);
            *connection_initialized = true;
        }
    }

    if state.is_connected {}
}
