use bevy::prelude::*;
use kilter_brain::{
    board_connection::{BoardDevice, Connect, NearbyBoards, StartScan, StopScan, WriteToBoard},
    kilter_data::KilterData,
};
use serde::{Deserialize, Serialize};
use std::ffi;

const SERVICE_UUID: &str = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E";
const CHARACTERISTIC_UUID: &str = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E";
// Our esp32 kilter board facsimile fails to process any more than 20 bytes at a time.
// It's not clear if this is a limitation of the facsimile, or if it's also an issue with
// a real board. However, the official kilter app also doesn't broach this limit when
// sending data to the facsimile.
const BLE_CHUNK_SIZE: usize = 20;

#[derive(Resource)]
pub struct ScanPollTimer(Timer);
impl Default for ScanPollTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.2, TimerMode::Repeating))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleState {
    pub is_on: bool,
    pub is_scanning: bool,
    pub is_connected: bool,
    pub devices: Vec<BleDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
    info!(
        "BLE: Writing to characteristic ({} bytes total):",
        data.len()
    );

    for (index, &byte) in data.iter().enumerate() {
        let ascii = if byte.is_ascii_graphic() || byte == b' ' {
            format!("'{}'", byte as char)
        } else {
            "·".to_string()
        };

        info!("BLE:  [{}]: 0x{:02X} ({:3}) {}", index, byte, byte, ascii);
    }

    let service_c_str = unsafe { ffi::CString::new(service_uuid).unwrap() };
    let characteristic_c_str = unsafe { ffi::CString::new(characteristic_uuid).unwrap() };

    for (chunk_index, chunk) in data.chunks(BLE_CHUNK_SIZE).enumerate() {
        info!("BLE: Writing chunk {} ({} bytes)", chunk_index, chunk.len());

        let success = unsafe {
            ble_write_characteristic(
                service_c_str.as_ptr(),
                characteristic_c_str.as_ptr(),
                chunk.as_ptr(),
                chunk.len(),
            )
        };

        if !success {
            info!("BLE: Failed to write chunk {}", chunk_index);
            return false;
        }
    }

    true
}

fn calculate_checksum(data: &[u8]) -> u8 {
    let mut i = 0;
    for &byte in data {
        i = (i + byte as i32) & 255;
    }
    ((!i) & 255) as u8
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

    packet
}

pub struct BlePlugin;
impl Plugin for BlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (scan_poll, start_scan, stop_scan, connect, write_to_board),
        );
        app.init_resource::<ScanPollTimer>();
    }
}

fn start_scan(mut events: EventReader<StartScan>) {
    if events.len() == 0 {
        return;
    }

    info!("BLE: start scan requested");

    unsafe { ble_start_scan() };

    events.clear();
}

fn stop_scan(mut events: EventReader<StopScan>) {
    if events.len() == 0 {
        return;
    }

    info!("BLE: stop scan requested");

    unsafe { ble_stop_scan() };

    events.clear();
}

fn connect(mut events: EventReader<Connect>) {
    for event in events.read() {
        connect_to_device(&event.device_id);
        unsafe { ble_stop_scan() };
    }
}

fn write_to_board(mut events: EventReader<WriteToBoard>) {
    for event in events.read() {
        let encoded = encode_holds_data(&event.0);
        write_to_characteristic(SERVICE_UUID, CHARACTERISTIC_UUID, &encoded);
    }
}

fn scan_poll(
    mut connection_initialized: Local<bool>,
    mut wrote_test_data: Local<bool>,
    mut delay: Local<u32>,
    kd: Res<KilterData>,
    mut timer: ResMut<ScanPollTimer>,
    time: Res<Time>,
    mut nearby_boards: ResMut<NearbyBoards>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let Some(state) = get_ble_state() else {
        info!("BLE: rust no state");
        return;
    };

    if state.is_scanning {
        info!("BLE: {:?}", state);
    }

    let mut boards = vec![];
    for device in state.devices {
        boards.push(BoardDevice {
            name: if device.advertised_name != "Unknown" {
                device.advertised_name
            } else {
                device.name
            },
            id: device.id,
        })
    }

    nearby_boards.set_if_neq(NearbyBoards(boards));

    // for device in state.devices {
    //     if device.advertised_name.starts_with("Fake Kilter")
    //         && !*connection_initialized
    //         && !state.is_connected
    //     {
    //         info!("BLE: rust wants to connect to Fake Kilter");
    //         connect_to_device(&device.id);
    //         *connection_initialized = true;
    //     }
    // }

    // if state.is_connected && !*wrote_test_data && *delay < 5 {
    //     info!("BLE: Delaying");
    //     *delay += 1;
    // }

    // if state.is_connected && !*wrote_test_data && *delay >= 5 {
    //     info!("BLE: rust knows we're connected. Writing test data");

    //     let placement_color = [
    //         (1145, (255, 0, 0)),
    //         (1146, (255, 0, 0)),
    //         (1149, (255, 0, 0)),
    //         (1186, (255, 0, 0)),
    //     ];

    //     let position_color = placement_color
    //         .iter()
    //         .flat_map(|(placement, color)| {
    //             let Some(position) = kd.placement_id_to_led_position.get(placement) else {
    //                 return None;
    //             };
    //             Some((*position as u16, *color))
    //         })
    //         .collect::<Vec<_>>();

    //     let encoded = encode_holds_data(&position_color);
    //     write_to_characteristic(SERVICE_UUID, CHARACTERISTIC_UUID, &encoded);
    //     *wrote_test_data = true;
    // }
}
