## Colibri R
Found BLE peripheral : 'Some("Neurotech_Colibri_R")' : address = [82, 173, 134, 228, 112, 224] is connected: false
start connect to peripheral : Some("Neurotech_Colibri_R") = [82, 173, 134, 228, 112, 224]...
now connected ('true') to peripheral : Some("Neurotech_Colibri_R")...
Discover peripheral : 'Some("Neurotech_Colibri_R")' characteristics...
Characteristic { start_handle: 2, end_handle: 65535, value_handle: 3, uuid: 2A:00, properties: READ | WRITE }
Characteristic { start_handle: 4, end_handle: 65535, value_handle: 5, uuid: 2A:01, properties: READ }
Characteristic { start_handle: 6, end_handle: 65535, value_handle: 7, uuid: 2A:04, properties: READ }
Characteristic { start_handle: 10, end_handle: 65535, value_handle: 11, uuid: 3D:2F:00:03:D6:B9:11:E4:88:CF:00:02:A5:D5:C5:1B, properties: NOTIFY }
Characteristic { start_handle: 13, end_handle: 65535, value_handle: 14, uuid: 3D:2F:00:02:D6:B9:11:E4:88:CF:00:02:A5:D5:C5:1B, properties: WRITE_WITHOUT_RESPONSE | WRITE }
disconnecting from peripheral : Some("Neurotech_Colibri_R")...

## BrainBit
Found BLE peripheral : 'Some("NeuroBLE")' : address = [71, 19, 148, 10, 107, 230] is connected: false
start connect to peripheral : Some("NeuroBLE") = [71, 19, 148, 10, 107, 230]...
now connected ('true') to peripheral : Some("NeuroBLE")...
Discover peripheral : 'Some("NeuroBLE")' characteristics...
Characteristic { start_handle: 2, end_handle: 65535, value_handle: 3, uuid: 2A:00, properties: READ | WRITE }
Characteristic { start_handle: 4, end_handle: 65535, value_handle: 5, uuid: 2A:01, properties: READ }
Characteristic { start_handle: 6, end_handle: 65535, value_handle: 7, uuid: 2A:04, properties: READ }
Characteristic { start_handle: 10, end_handle: 65535, value_handle: 11, uuid: 6E:40:00:02:B5:34:F3:93:67:A9:E5:0E:24:DC:CA:9E, properties: NOTIFY }
Characteristic { start_handle: 14, end_handle: 65535, value_handle: 15, uuid: 2A:19, properties: READ | NOTIFY }
disconnecting from peripheral : Some("NeuroBLE")...


Found BLE peripheral : 'Some("BrainBit")' : address = [209, 151, 75, 13, 199, 241] is connected: false
start connect to peripheral : Some("BrainBit") = [209, 151, 75, 13, 199, 241]...
now connected ('true') to peripheral : Some("BrainBit")...
Discover peripheral : 'Some("BrainBit")' characteristics...
Characteristic { start_handle: 2, end_handle: 65535, value_handle: 3, uuid: 2A:00, properties: READ | WRITE }
Characteristic { start_handle: 4, end_handle: 65535, value_handle: 5, uuid: 2A:01, properties: READ }
Characteristic { start_handle: 6, end_handle: 65535, value_handle: 7, uuid: 2A:04, properties: READ }
Characteristic { start_handle: 9, end_handle: 65535, value_handle: 10, uuid: 2A:05, properties: INDICATE }
Characteristic { start_handle: 13, end_handle: 65535, value_handle: 14, uuid: 6E:40:00:02:B5:34:F3:93:68:A9:E5:0E:24:DC:CA:9E, properties: READ | NOTIFY }
Characteristic { start_handle: 16, end_handle: 65535, value_handle: 17, uuid: 6E:40:00:03:B5:34:F3:93:68:A9:E5:0E:24:DC:CA:9E, properties: WRITE }
Characteristic { start_handle: 18, end_handle: 65535, value_handle: 19, uuid: 6E:40:00:04:B5:34:F3:93:68:A9:E5:0E:24:DC:CA:9E, properties: NOTIFY }
Characteristic { start_handle: 22, end_handle: 65535, value_handle: 23, uuid: 2A:24, properties: READ }
Characteristic { start_handle: 24, end_handle: 65535, value_handle: 25, uuid: 2A:25, properties: READ }
Characteristic { start_handle: 26, end_handle: 65535, value_handle: 27, uuid: 2A:27, properties: READ }
Characteristic { start_handle: 28, end_handle: 65535, value_handle: 29, uuid: 2A:26, properties: READ }
disconnecting from peripheral : Some("BrainBit")...

## OnEvent
DeviceDiscovered: 54:BD:79:23:44:07
DeviceDiscovered: E0:70:E4:86:AD:52
DeviceDiscovered: 9C:8C:6E:10:7D:60
DeviceDiscovered: E6:38:AA:A2:87:43
Count = 2
DeviceDiscovered: E0:70:E4:86:AD:52 // ColibriR
Count = 3
Count = 8
DeviceDiscovered: E6:6B:0A:94:13:47 // BB - old


Win10 - subscribe run
===================
Peripheral "BrainBit" is connected: false
Found matching peripheral "BrainBit"...
Now connected (true) to peripheral "BrainBit".
Discover peripheral "BrainBit" services...
Checking characteristic Characteristic { uuid: 00002a00-0000-1000-8000-00805f9b34fb, service_uuid: 00001800-0000-1000-8000-00805f9b34fb, properties: READ | WRITE }
Checking characteristic Characteristic { uuid: 00002a01-0000-1000-8000-00805f9b34fb, service_uuid: 00001800-0000-1000-8000-00805f9b34fb, properties: READ }
Checking characteristic Characteristic { uuid: 00002a04-0000-1000-8000-00805f9b34fb, service_uuid: 00001800-0000-1000-8000-00805f9b34fb, properties: READ }
Checking characteristic Characteristic { uuid: 00002a05-0000-1000-8000-00805f9b34fb, service_uuid: 00001801-0000-1000-8000-00805f9b34fb, properties: INDICATE }
Checking characteristic Characteristic { uuid: 00002a24-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ }
Checking characteristic Characteristic { uuid: 00002a25-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ }
Checking characteristic Characteristic { uuid: 00002a26-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ }
Checking characteristic Characteristic { uuid: 00002a27-0000-1000-8000-00805f9b34fb, service_uuid: 0000180a-0000-1000-8000-00805f9b34fb, properties: READ }
Checking characteristic Characteristic { uuid: 6e400002-b534-f393-68a9-e50e24dcca9e, service_uuid: 6e400001-b534-f393-68a9-e50e24dcca9e, properties: READ | NOTIFY }
Checking characteristic Characteristic { uuid: 6e400003-b534-f393-68a9-e50e24dcca9e, service_uuid: 6e400001-b534-f393-68a9-e50e24dcca9e, properties: WRITE }
Checking characteristic Characteristic { uuid: 6e400004-b534-f393-68a9-e50e24dcca9e, service_uuid: 6e400001-b534-f393-68a9-e50e24dcca9e, properties: NOTIFY }
Disconnecting from peripheral "BrainBit"...
Peripheral "(peripheral name unknown)" is connected: false
