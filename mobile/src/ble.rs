use bevy::{platform::collections::HashMap, prelude::*};
use kilter_brain::kilter_data::KilterData;
use serde::{Deserialize, Serialize};
use std::ffi;

const SERVICE_UUID: &str = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E";
const CHARACTERISTIC_UUID: &str = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E";

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
    // Log the data being written in hex format
    let hex_string = data
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ");
    info!(
        "BLE: Writing to characteristic ({} bytes): {}",
        data.len(),
        hex_string
    );

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

fn calculate_checksum(data: &[u8]) -> u8 {
    let sum = data.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte));
    (!sum) & 0xFF
}

pub fn encode_holds_data_level_2(holds: &[(u16, (u8, u8, u8))]) -> Vec<u8> {
    let mut packet_data = Vec::new();

    // Single packet marker (API level 2)
    packet_data.push(80); // 'P' - single packet

    // Encode each hold as 2 bytes
    for &(position, (r, g, b)) in holds {
        // Compress 8-bit RGB to 2 bits each (6 bits total)
        let r_compressed = (r >> 6) & 0x03;
        let g_compressed = (g >> 6) & 0x03;
        let b_compressed = (b >> 6) & 0x03;

        // First byte: lowest 8 bits of position
        let byte1 = (position & 0xFF) as u8;

        // Second byte: highest 2 bits of position + 6 bits of RGB
        let byte2 = ((position >> 8) & 0x03) as u8
            | (r_compressed << 6)
            | (g_compressed << 4)
            | (b_compressed << 2);

        packet_data.push(byte1);
        packet_data.push(byte2);
    }

    // Build complete packet
    let mut packet = Vec::new();
    packet.push(1); // First byte always 1
    packet.push(packet_data.len() as u8); // Size of packet data
    packet.push(calculate_checksum(&packet_data)); // Checksum
    packet.push(2); // Fourth byte always 2
    packet.extend_from_slice(&packet_data); // Packet data
    packet.push(3); // Final byte always 3

    // Log the encoded data in hex format
    let hex_string = packet
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ");
    info!(
        "BLE: Encoded packet ({} bytes): {}",
        packet.len(),
        hex_string
    );

    packet
}

pub fn encode_holds_data(holds: &[(u16, (u8, u8, u8))]) -> Vec<u8> {
    let mut packet_data = Vec::new();

    // Single packet marker (API level 3)
    packet_data.push(84); // 'T' - single packet

    // Encode each hold as 3 bytes
    for &(position, (r, g, b)) in holds {
        // Compress RGB: R and G to 3 bits, B to 2 bits
        let r_compressed = (r >> 5) & 0x07; // 3 bits
        let g_compressed = (g >> 5) & 0x07; // 3 bits
        let b_compressed = (b >> 6) & 0x03; // 2 bits

        // First byte: lowest 8 bits of position
        let byte1 = (position & 0xFF) as u8;

        // Second byte: highest 8 bits of position
        let byte2 = ((position >> 8) & 0xFF) as u8;

        // Third byte: RGB color (3R + 3G + 2B = 8 bits)
        let byte3 = (r_compressed << 5) | (g_compressed << 2) | b_compressed;

        packet_data.push(byte1);
        packet_data.push(byte2);
        packet_data.push(byte3);
    }

    // Build complete packet
    let mut packet = Vec::new();
    packet.push(1); // First byte always 1
    packet.push(packet_data.len() as u8); // Size of packet data
    packet.push(calculate_checksum(&packet_data)); // Checksum
    packet.push(2); // Fourth byte always 2
    packet.extend_from_slice(&packet_data); // Packet data
    packet.push(3); // Final byte always 3

    // Log the encoded data in hex format
    let hex_string = packet
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ");
    info!(
        "BLE: Encoded packet Level 3 ({} bytes): {}",
        packet.len(),
        hex_string
    );

    packet
}

pub struct BlePlugin;
impl Plugin for BlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update);
        app.insert_resource(Time::<Fixed>::from_seconds(1.0));
    }
}

fn update(
    mut connection_initialized: Local<bool>,
    mut wrote_test_data: Local<bool>,
    mut delay: Local<u32>,
    kd: Res<KilterData>,
) {
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

    if state.is_connected && !*wrote_test_data && *delay < 5 {
        info!("BLE: Delaying");
        *delay += 1;
    }

    if state.is_connected && !*wrote_test_data && *delay >= 5 {
        info!("BLE: rust knows we're connected. Writing test data");

        let placement_color = [
            (1145, (255, 0, 0)),
            (1146, (255, 0, 0)),
            (1149, (255, 0, 0)),
            (1186, (255, 0, 0)),
        ];

        let position_color = placement_color
            .iter()
            .flat_map(|(placement, color)| {
                let Some(position) = kd.placement_id_to_led_position.get(placement) else {
                    return None;
                };
                Some((*position as u16, *color))
            })
            .collect::<Vec<_>>();

        let encoded = encode_holds_data(&position_color);
        write_to_characteristic(SERVICE_UUID, CHARACTERISTIC_UUID, &encoded);
        *wrote_test_data = true;
    }
}
