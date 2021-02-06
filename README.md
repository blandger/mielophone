# Mielophone
Funny research project about BLE device(s) functionality and data gathering.
[![Build Status](https://travis-ci.com/blandger/mielophone.svg?branch=master)](https://travis-ci.com/blandger/mielophone)

## Linux
Usually console app should be run with 'sudo' privileges.

You can check if any BLE devices are present on PC, output peripherals to console like:
https://unix.stackexchange.com/questions/96106/bluetooth-le-scan-as-non-root
> sudo apt-get install libcap2-bin

#### Run BLE app without sudo privileges on Linux
There is another receipt to run linux command for binary app like:

> sudo setcap 'cap_net_raw,cap_net_admin+eip' /absolute/path/to/your/executable/file

>sudo setcap 'cap_net_raw,cap_net_admin+eip' XXXX/mielophone/target/debug/console

Unfortunately you have to run it after every app rebuild. 

### Packages to install
> sudo apt-get install clang

> sudo apt-get install libdbus-1-dev

OR

> sudo apt-get install librust-libdbus-sys-dev

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
