import Foundation
import CoreBluetooth

@_cdecl("ble_start_scan")
public func ble_start_scan() {
    BleManager.shared.scan()
}

@_cdecl("ble_stop_scan")
public func ble_stop_scan() {
    BleManager.shared.stop()
}

public struct BleDevice {
    public let name: String
    public let identifier: UUID
}

@_cdecl("ble_get_state_json")
public func ble_get_state_json() -> UnsafePointer<CChar>? {
    let state = BleManager.shared.poll()

    // Build a C-compatible JSON string
    let deviceList = state.discovered.map {
        [
            "name": $0.name ?? "Unknown",
            "advertised_name": BleManager.shared.peripheralAdvertisedNames[$0.identifier] ?? "Unknown",
            "id": $0.identifier.uuidString
        ]
    }

    let dict: [String: Any] = [
        "is_on": state.isOn,
        "is_scanning": state.isScanning,
        "devices": deviceList
    ]

    guard let jsonData = try? JSONSerialization.data(withJSONObject: dict, options: []),
          let jsonStr = String(data: jsonData, encoding: .utf8) else {
        return nil
    }

    // Allocate a C string
    let cString = strdup(jsonStr)
    return UnsafePointer(cString)
}

@_cdecl("ble_free_string")
public func ble_free_string(_ ptr: UnsafePointer<CChar>?) {
    if let ptr = ptr {
        free(UnsafeMutableRawPointer(mutating: ptr))
    }
}

struct BleState {
    let isOn: Bool
    let isScanning: Bool
    let discovered: [CBPeripheral]
}

class BleManager: NSObject, CBCentralManagerDelegate, @unchecked Sendable {
    static let shared = BleManager()

    private var centralManager: CBCentralManager!
    private(set) var isOn: Bool = false
    private(set) var isScanning: Bool = false
    private(set) var discoveredPeripherals: [CBPeripheral] = []
    private(set) var peripheralAdvertisedNames: [UUID: String] = [:]

    private override init() {
        super.init()
        centralManager = CBCentralManager(delegate: self, queue: nil)
    }

    func scan() {
        guard isOn else {
            print("Bluetooth not powered on.")
            return
        }

        if !isScanning {
            discoveredPeripherals.removeAll()
            peripheralAdvertisedNames.removeAll()
            let targetServiceUUID = CBUUID(string: "4488B571-7806-4DF6-BCFF-A2897E4953FF") // Replace as needed
            centralManager.scanForPeripherals(withServices: [targetServiceUUID], options: nil)
            isScanning = true
        }
    }

    func stop() {
        if isScanning {
            centralManager.stopScan()
            isScanning = false
        }
    }

    func poll() -> BleState {
        return BleState(
            isOn: isOn,
            isScanning: isScanning,
            discovered: discoveredPeripherals
        )
    }

    // MARK: - CBCentralManagerDelegate

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        isOn = (central.state == .poweredOn)
        if !isOn {
            isScanning = false
            discoveredPeripherals.removeAll()
            peripheralAdvertisedNames.removeAll()
        }
    }

    func centralManager(_ central: CBCentralManager,
                        didDiscover peripheral: CBPeripheral,
                        advertisementData: [String : Any],
                        rssi RSSI: NSNumber) {

        // Avoid duplicates
        if !discoveredPeripherals.contains(where: { $0.identifier == peripheral.identifier }) {
            discoveredPeripherals.append(peripheral)
        }

        if let advertisedName = advertisementData[CBAdvertisementDataLocalNameKey] as? String {
            peripheralAdvertisedNames[peripheral.identifier] = advertisedName
        }
    }
}
