# Mielophone
Funny research project about BLE device(s) functionality and data gathering.
[![Build Status](https://travis-ci.com/blandger/mielophone.svg?branch=master)](https://travis-ci.com/blandger/mielophone)

## Linux
Console app should be run with 'sudo' privileges now

You can check if any BLE devices are present on PC, output peripherals to console like:
https://unix.stackexchange.com/questions/96106/bluetooth-le-scan-as-non-root
> sudo apt-get install libcap2-bin

### Packages to install
> sudo apt-get install clang
 

## Colibri R
Discover peripheral : 'Some("Neurotech_Colibri_R")' characteristics...
Characteristic { start_handle: 2, end_handle: 65535, value_handle: 3, uuid: 2A:00, properties: READ | WRITE }
Characteristic { start_handle: 4, end_handle: 65535, value_handle: 5, uuid: 2A:01, properties: READ }
Characteristic { start_handle: 6, end_handle: 65535, value_handle: 7, uuid: 2A:04, properties: READ }
TX channel { start_handle: 13, end_handle: 65535, value_handle: 14, uuid: 3D:2F:00:02:D6:B9:11:E4:88:CF:00:02:A5:D5:C5:1B, properties: WRITE_WITHOUT_RESPONSE | WRITE 
RX channel { start_handle: 10, end_handle: 65535, value_handle: 11, uuid: 3D:2F:00:03:D6:B9:11:E4:88:CF:00:02:A5:D5:C5:1B, properties: NOTIFY }
