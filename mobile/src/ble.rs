use bevy::{platform::collections::HashMap, prelude::*};
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

pub fn get_position_from_placement(placement: u32) -> Option<u16> {
    let positions: HashMap<u32, u16> = HashMap::from([
        (1447, 0),
        (1073, 1),
        (1448, 2),
        (1074, 3),
        (1449, 4),
        (1075, 5),
        (1450, 6),
        (1076, 7),
        (1451, 8),
        (1077, 9),
        (1452, 10),
        (1078, 11),
        (1453, 12),
        (1079, 13),
        (1454, 14),
        (1080, 15),
        (1455, 16),
        (1081, 17),
        (1456, 18),
        (1082, 19),
        (1457, 20),
        (1083, 21),
        (1458, 22),
        (1084, 23),
        (1459, 24),
        (1085, 25),
        (1460, 26),
        (1086, 27),
        (1461, 28),
        (1087, 29),
        (1462, 30),
        (1088, 31),
        (1463, 32),
        (1464, 33),
        (1089, 34),
        (1090, 36),
        (1465, 37),
        (1107, 38),
        (1474, 39),
        (1124, 40),
        (1483, 41),
        (1141, 42),
        (1492, 43),
        (1158, 44),
        (1501, 45),
        (1175, 46),
        (1510, 47),
        (1192, 48),
        (1519, 49),
        (1209, 50),
        (1528, 51),
        (1226, 52),
        (1537, 53),
        (1243, 54),
        (1546, 55),
        (1260, 56),
        (1555, 57),
        (1277, 58),
        (1564, 59),
        (1294, 60),
        (1573, 61),
        (1311, 62),
        (1582, 63),
        (1328, 64),
        (1591, 65),
        (1345, 66),
        (1362, 67),
        (1379, 68),
        (1380, 69),
        (1363, 70),
        (1346, 71),
        (1592, 72),
        (1329, 73),
        (1312, 74),
        (1574, 75),
        (1295, 76),
        (1278, 77),
        (1556, 78),
        (1261, 79),
        (1244, 80),
        (1538, 81),
        (1227, 82),
        (1210, 83),
        (1520, 84),
        (1193, 85),
        (1176, 86),
        (1502, 87),
        (1159, 88),
        (1142, 89),
        (1484, 90),
        (1125, 91),
        (1108, 92),
        (1466, 93),
        (1091, 94),
        (1092, 95),
        (1109, 96),
        (1475, 97),
        (1126, 98),
        (1143, 99),
        (1493, 100),
        (1160, 101),
        (1177, 102),
        (1511, 103),
        (1194, 104),
        (1211, 105),
        (1529, 106),
        (1228, 107),
        (1245, 108),
        (1547, 109),
        (1262, 110),
        (1279, 111),
        (1565, 112),
        (1296, 113),
        (1313, 114),
        (1583, 115),
        (1330, 116),
        (1347, 117),
        (1364, 118),
        (1381, 119),
        (1382, 120),
        (1365, 121),
        (1348, 122),
        (1593, 123),
        (1331, 124),
        (1314, 125),
        (1575, 126),
        (1297, 127),
        (1280, 128),
        (1557, 129),
        (1263, 130),
        (1246, 131),
        (1539, 132),
        (1229, 133),
        (1212, 134),
        (1521, 135),
        (1195, 136),
        (1178, 137),
        (1503, 138),
        (1161, 139),
        (1144, 140),
        (1485, 141),
        (1127, 142),
        (1110, 143),
        (1467, 144),
        (1093, 145),
        (1094, 146),
        (1111, 147),
        (1476, 148),
        (1128, 149),
        (1145, 150),
        (1494, 151),
        (1162, 152),
        (1179, 153),
        (1512, 154),
        (1196, 155),
        (1213, 156),
        (1530, 157),
        (1230, 158),
        (1247, 159),
        (1548, 160),
        (1264, 161),
        (1281, 162),
        (1566, 163),
        (1298, 164),
        (1315, 165),
        (1584, 166),
        (1332, 167),
        (1349, 168),
        (1366, 169),
        (1383, 170),
        (1384, 171),
        (1367, 172),
        (1350, 173),
        (1594, 174),
        (1333, 175),
        (1316, 176),
        (1576, 177),
        (1299, 178),
        (1282, 179),
        (1558, 180),
        (1265, 181),
        (1248, 182),
        (1540, 183),
        (1231, 184),
        (1214, 185),
        (1522, 186),
        (1197, 187),
        (1180, 188),
        (1504, 189),
        (1163, 190),
        (1146, 191),
        (1486, 192),
        (1129, 193),
        (1112, 194),
        (1468, 195),
        (1095, 196),
        (1096, 197),
        (1113, 198),
        (1477, 199),
        (1130, 200),
        (1147, 201),
        (1495, 202),
        (1164, 203),
        (1181, 204),
        (1513, 205),
        (1198, 206),
        (1215, 207),
        (1531, 208),
        (1232, 209),
        (1249, 210),
        (1549, 211),
        (1266, 212),
        (1283, 213),
        (1567, 214),
        (1300, 215),
        (1317, 216),
        (1585, 217),
        (1334, 218),
        (1351, 219),
        (1368, 220),
        (1385, 221),
        (1386, 222),
        (1369, 223),
        (1352, 224),
        (1595, 225),
        (1335, 226),
        (1318, 227),
        (1577, 228),
        (1301, 229),
        (1284, 230),
        (1559, 231),
        (1267, 232),
        (1250, 233),
        (1541, 234),
        (1233, 235),
        (1216, 236),
        (1523, 237),
        (1199, 238),
        (1182, 239),
        (1505, 240),
        (1165, 241),
        (1148, 242),
        (1487, 243),
        (1131, 244),
        (1114, 245),
        (1469, 246),
        (1097, 247),
        (1098, 248),
        (1115, 249),
        (1478, 250),
        (1132, 251),
        (1149, 252),
        (1496, 253),
        (1166, 254),
        (1183, 255),
        (1514, 256),
        (1200, 257),
        (1217, 258),
        (1532, 259),
        (1234, 260),
        (1251, 261),
        (1550, 262),
        (1268, 263),
        (1285, 264),
        (1568, 265),
        (1302, 266),
        (1319, 267),
        (1586, 268),
        (1336, 269),
        (1353, 270),
        (1370, 271),
        (1387, 272),
        (1388, 273),
        (1371, 274),
        (1354, 275),
        (1596, 276),
        (1337, 277),
        (1320, 278),
        (1578, 279),
        (1303, 280),
        (1286, 281),
        (1560, 282),
        (1269, 283),
        (1252, 284),
        (1542, 285),
        (1235, 286),
        (1218, 287),
        (1524, 288),
        (1201, 289),
        (1184, 290),
        (1506, 291),
        (1167, 292),
        (1150, 293),
        (1488, 294),
        (1133, 295),
        (1116, 296),
        (1470, 297),
        (1099, 298),
        (1100, 299),
        (1117, 300),
        (1479, 301),
        (1134, 302),
        (1151, 303),
        (1497, 304),
        (1168, 305),
        (1185, 306),
        (1515, 307),
        (1202, 308),
        (1219, 309),
        (1533, 310),
        (1236, 311),
        (1253, 312),
        (1551, 313),
        (1270, 314),
        (1287, 315),
        (1569, 316),
        (1304, 317),
        (1321, 318),
        (1587, 319),
        (1338, 320),
        (1355, 321),
        (1372, 322),
        (1389, 323),
        (1390, 324),
        (1373, 325),
        (1356, 326),
        (1597, 327),
        (1339, 328),
        (1322, 329),
        (1579, 330),
        (1305, 331),
        (1288, 332),
        (1561, 333),
        (1271, 334),
        (1254, 335),
        (1543, 336),
        (1237, 337),
        (1220, 338),
        (1525, 339),
        (1203, 340),
        (1186, 341),
        (1507, 342),
        (1169, 343),
        (1152, 344),
        (1489, 345),
        (1135, 346),
        (1118, 347),
        (1471, 348),
        (1101, 349),
        (1102, 350),
        (1119, 351),
        (1480, 352),
        (1136, 353),
        (1153, 354),
        (1498, 355),
        (1170, 356),
        (1187, 357),
        (1516, 358),
        (1204, 359),
        (1221, 360),
        (1534, 361),
        (1238, 362),
        (1255, 363),
        (1552, 364),
        (1272, 365),
        (1289, 366),
        (1570, 367),
        (1306, 368),
        (1323, 369),
        (1588, 370),
        (1340, 371),
        (1357, 372),
        (1374, 373),
        (1391, 374),
        (1392, 375),
        (1375, 376),
        (1358, 377),
        (1598, 378),
        (1341, 379),
        (1324, 380),
        (1580, 381),
        (1307, 382),
        (1290, 383),
        (1562, 384),
        (1273, 385),
        (1256, 386),
        (1544, 387),
        (1239, 388),
        (1222, 389),
        (1526, 390),
        (1205, 391),
        (1188, 392),
        (1508, 393),
        (1171, 394),
        (1154, 395),
        (1490, 396),
        (1137, 397),
        (1120, 398),
        (1472, 399),
        (1103, 400),
        (1104, 401),
        (1121, 402),
        (1481, 403),
        (1138, 404),
        (1155, 405),
        (1499, 406),
        (1172, 407),
        (1189, 408),
        (1517, 409),
        (1206, 410),
        (1223, 411),
        (1535, 412),
        (1240, 413),
        (1257, 414),
        (1553, 415),
        (1274, 416),
        (1291, 417),
        (1571, 418),
        (1308, 419),
        (1325, 420),
        (1589, 421),
        (1342, 422),
        (1359, 423),
        (1376, 424),
        (1393, 425),
        (1394, 426),
        (1377, 427),
        (1360, 428),
        (1599, 429),
        (1343, 430),
        (1326, 431),
        (1581, 432),
        (1309, 433),
        (1292, 434),
        (1563, 435),
        (1275, 436),
        (1258, 437),
        (1545, 438),
        (1241, 439),
        (1224, 440),
        (1527, 441),
        (1207, 442),
        (1190, 443),
        (1509, 444),
        (1173, 445),
        (1156, 446),
        (1491, 447),
        (1139, 448),
        (1122, 449),
        (1473, 450),
        (1105, 451),
        (1106, 452),
        (1123, 453),
        (1482, 454),
        (1140, 455),
        (1157, 456),
        (1500, 457),
        (1174, 458),
        (1191, 459),
        (1518, 460),
        (1208, 461),
        (1225, 462),
        (1536, 463),
        (1242, 464),
        (1259, 465),
        (1554, 466),
        (1276, 467),
        (1293, 468),
        (1572, 469),
        (1310, 470),
        (1327, 471),
        (1590, 472),
        (1344, 473),
        (1361, 474),
        (1378, 475),
        (1395, 476),
    ]);
    positions.get(&placement).cloned()
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
        let encoded = encode_holds_data(&[
            (get_position_from_placement(1145).unwrap(), (255, 0, 0)),
            (get_position_from_placement(1146).unwrap(), (255, 0, 0)),
            (get_position_from_placement(1149).unwrap(), (255, 0, 0)),
            (get_position_from_placement(1186).unwrap(), (255, 0, 0)),
        ]);
        write_to_characteristic(SERVICE_UUID, CHARACTERISTIC_UUID, &encoded);
        *wrote_test_data = true;
    }
}
