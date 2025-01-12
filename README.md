# Mielophone
Funny research project about BLE device(s) functionality and data gathering.

![Actions Build Status](https://github.com/blandger/mielophone/actions/workflows/rust.yml/badge.svg?branch=master)

## Linux
Usually console app should be run with 'sudo' privileges.

You can check if any BLE devices are present on PC, output peripherals to console like:
https://unix.stackexchange.com/questions/96106/bluetooth-le-scan-as-non-root
> sudo apt-get install libcap2-bin

#### Run BLE app without sudo privileges on Linux
There is another receipt to run linux command for binary app like:

>sudo setcap 'cap_net_raw,cap_net_admin+eip' ./path/to/your/executable/file

>sudo setcap 'cap_net_raw,cap_net_admin+eip' ./target/debug/battery_level

Unfortunately you have to run it after every app rebuild. 

### Packages to install
> sudo apt-get install clang

> sudo apt-get install libdbus-1-dev

OR

> sudo apt-get install librust-libdbus-sys-dev
