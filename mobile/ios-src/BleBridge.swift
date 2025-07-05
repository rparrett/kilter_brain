import CoreBluetooth
import Foundation

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
            "advertised_name": BleManager.shared.peripheralAdvertisedNames[
                $0.identifier] ?? "Unknown",
            "id": $0.identifier.uuidString,
        ]
    }

    let dict: [String: Any] = [
        "is_on": state.isOn,
        "is_scanning": state.isScanning,
        "is_connected": state.isConnected,
        "devices": deviceList,
    ]

    guard
        let jsonData = try? JSONSerialization.data(
            withJSONObject: dict, options: []),
        let jsonStr = String(data: jsonData, encoding: .utf8)
    else {
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

@_cdecl("ble_connect")
public func ble_connect(_ idStr: UnsafePointer<CChar>) -> Bool {
    let uuidString = String(cString: idStr)
    guard let uuid = UUID(uuidString: uuidString) else {
        print("BLE: Invalid UUID string: \(uuidString)")
        return false
    }

    return BleManager.shared.connect(to: uuid)
}

@_cdecl("ble_disconnect")
public func ble_disconnect() -> Bool {
    print("BLE: disconnect")
    return BleManager.shared.disconnect()
}

@_cdecl("ble_write_characteristic")
public func ble_write_characteristic(
    _ serviceUUID: UnsafePointer<CChar>,
    _ characteristicUUID: UnsafePointer<CChar>,
    _ data: UnsafePointer<UInt8>,
    _ dataLength: Int
) -> Bool {
    let serviceUUIDString = String(cString: serviceUUID)
    let characteristicUUIDString = String(cString: characteristicUUID)
    let dataToWrite = Data(bytes: data, count: dataLength)

    return BleManager.shared.writeToCharacteristic(
        serviceUUID: serviceUUIDString,
        characteristicUUID: characteristicUUIDString,
        data: dataToWrite
    )
}

struct BleState {
    let isOn: Bool
    let isScanning: Bool
    let isConnected: Bool
    let discovered: [CBPeripheral]
}

class BleManager: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate,
    @unchecked Sendable
{
    static let shared = BleManager()

    private var centralManager: CBCentralManager!
    private(set) var isOn: Bool = false
    private(set) var isScanning: Bool = false
    private(set) var discoveredPeripherals: [CBPeripheral] = []
    private(set) var peripheralAdvertisedNames: [UUID: String] = [:]
    private(set) var allPeripherals: [UUID: CBPeripheral] = [:]
    private var connectedPeripheral: CBPeripheral?

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
            let targetServiceUUID = CBUUID(
                string: "4488B571-7806-4DF6-BCFF-A2897E4953FF")  // Replace as needed
            centralManager.scanForPeripherals(
                withServices: [targetServiceUUID], options: nil)
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
            isConnected: connectedPeripheral != nil,
            discovered: discoveredPeripherals
        )
    }

    func getPeripheral(by id: UUID) -> CBPeripheral? {
        return allPeripherals[id]
    }

    func connect(to id: UUID) -> Bool {
        guard let peripheral = allPeripherals[id] else {
            print("BLE: Peripheral with ID \(id) not found")
            return false
        }

        print("BLE: Connecting to peripheral \(id)")
        peripheral.delegate = self
        centralManager.connect(peripheral, options: nil)
        return true
    }

    func disconnect() -> Bool {
        guard let peripheral = connectedPeripheral else {
            print("BLE: No connected peripheral to disconnect")
            return false
        }

        print("BLE: Disconnecting from peripheral \(peripheral.identifier)")
        centralManager.cancelPeripheralConnection(peripheral)
        return true
    }

    func writeToCharacteristic(
        serviceUUID: String, characteristicUUID: String, data: Data
    ) -> Bool {
        guard let peripheral = connectedPeripheral else {
            print("BLE: No connected peripheral")
            return false
        }

        guard
            let service = peripheral.services?.first(where: {
                $0.uuid == CBUUID(string: serviceUUID)
            })
        else {
            print("BLE: Service \(serviceUUID) not found")
            return false
        }

        guard
            let characteristic = service.characteristics?.first(where: {
                $0.uuid == CBUUID(string: characteristicUUID)
            })
        else {
            print("BLE: Characteristic \(characteristicUUID) not found")
            return false
        }

        print(
            "BLE: Writing \(data.count) bytes to characteristic \(characteristicUUID)"
        )
        peripheral.writeValue(data, for: characteristic, type: .withResponse)
        return true
    }

    // MARK: - CBCentralManagerDelegate

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        isOn = (central.state == .poweredOn)
        if !isOn {
            isScanning = false
            discoveredPeripherals.removeAll()
            peripheralAdvertisedNames.removeAll()
            allPeripherals.removeAll()
            connectedPeripheral = nil
        }
    }

    func centralManager(
        _ central: CBCentralManager,
        didDiscover peripheral: CBPeripheral,
        advertisementData: [String: Any],
        rssi RSSI: NSNumber
    ) {

        // Store peripheral persistently
        allPeripherals[peripheral.identifier] = peripheral

        // Avoid duplicates in current scan
        if !discoveredPeripherals.contains(where: {
            $0.identifier == peripheral.identifier
        }) {
            discoveredPeripherals.append(peripheral)
        }

        if let advertisedName = advertisementData[
            CBAdvertisementDataLocalNameKey] as? String
        {
            peripheralAdvertisedNames[peripheral.identifier] = advertisedName
        }
    }

    func centralManager(
        _ central: CBCentralManager, didConnect peripheral: CBPeripheral
    ) {
        print("BLE: Connected to peripheral \(peripheral.identifier)")
        connectedPeripheral = peripheral
        peripheral.discoverServices(nil)
    }

    func centralManager(
        _ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral,
        error: Error?
    ) {
        print(
            "BLE: Failed to connect to peripheral \(peripheral.identifier): \(error?.localizedDescription ?? "Unknown error")"
        )
    }

    func centralManager(
        _ central: CBCentralManager,
        didDisconnectPeripheral peripheral: CBPeripheral, error: Error?
    ) {
        print("BLE: Disconnected from peripheral \(peripheral.identifier)")
        if connectedPeripheral?.identifier == peripheral.identifier {
            connectedPeripheral = nil
        }
    }

    // MARK: - CBPeripheralDelegate

    func peripheral(
        _ peripheral: CBPeripheral, didDiscoverServices error: Error?
    ) {
        guard let services = peripheral.services else { return }

        for service in services {
            print("BLE: Discovered service: \(service.uuid)")
            peripheral.discoverCharacteristics(nil, for: service)
        }
    }

    func peripheral(
        _ peripheral: CBPeripheral,
        didDiscoverCharacteristicsFor service: CBService, error: Error?
    ) {
        guard let characteristics = service.characteristics else { return }

        for characteristic in characteristics {
            print(
                "BLE: Discovered characteristic: \(characteristic.uuid) for service: \(service.uuid)"
            )
        }
    }
}
