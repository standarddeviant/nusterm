
# nusterm

## Motivation
I recently started to use 'Serial over BLE' or 'Serial over Bluetooth' for work.
'Serial over Bluetooth' can mean a few things, but in this case it means 'NUS' or 'Nordic UART Service'.
To be explicit, it's software that uses this BLE Service and Characteristics:
```rust
const UART_SERVICE_UUID: Uuid = Uuid::from_u128(0x6E400001_B5A3_F393_E0A9_E50E24DCCA9E);
const UART_RX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400002_B5A3_F393_E0A9_E50E24DCCA9E);
const UART_TX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400003_B5A3_F393_E0A9_E50E24DCCA9E);
```
***NOTE: 'TX' and 'RX' in the characteristic names above are from the perspective of the
BLE Peripheral or the 'Device', so***
- TX = Peripheral-to-Central, i.e. BLE notifications
- RX = Central-to-Peripheral, i.e. BLE writes

### Existing Options
There are multiple options for BLE Serial client software including:
- `bleak` example script
  - great for scripting!
- Maker Diary's web-based NUS terminal
  - cool use of technology and super convenient
- Adafruit's phone app
  - reliable way to issue NUS commands from a phone
- Nordic's phone app

I started using `bleak` and the scripting it enables is just wonderful,
including the example script which is an excellent tool on its own.
While a great tool, the drawbacks I noticed were:
* something about python + async + BLE seems to be slow sometimes
* as always, for others to run this code they need to have python + bleak installed

### New Option (this tool)
I've been learning rust recently and was able to run the examples from `btleplug` successfully.
So with a bit of weekend hacking and learning about `tokio`, I got a good proof-of-concept working.
I've simplified it a bit and added some (currently fixed) colors to make it easy-ish to see what's
sent to the device and what came back as a response.

It's not perfect, and I'd like to make it a GUI at some point, but GUIs introduce complexity.
For now "colored text in the terminal" seems 'good enough' to put out to a wider audience and get
some feedback.

I hope people find this tool useful and expand on its capabilities.
Enjoy!

# Getting Started

To build the code, change to the `nusterm` directory on a linux system and run
```
pwsh ./nix_build.ps1
```

or 

To build the code, change to the `nusterm` directory on a windows system and run
```
pwsh .\win_build.ps1
```

This should produce a new binary of the current git hash in the `./bin` directory.

***NOTE: I don't have an OSX machine to test with, but would be happy for someone to test + contribute a build script for OSX***

## Build Prerequisites

The build scripts are simple, but require
* `Powershell 7+` via https://github.com/PowerShell/PowerShell/releases/latest
* `Rust` via https://rustup.rs/
* ?

## Authors

* **Dave Crist**

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Acknowledgments

* Thanks to the developers of `rust`, `tokio`, `btleplug`, and `rustyline` for making the tools I used to make this tool.

