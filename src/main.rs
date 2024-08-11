// Create a default reedline object to handle user input

use std::io;
use std::error::Error;
use std::time::Duration;

use text_io::read;
use anyhow;

use clap::Parser;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral as ApiPeripheral, ScanFilter, Characteristic, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral as PlatformPeripheral};
use futures::stream::StreamExt;

use tokio::time;
use uuid::Uuid;


use inquire::Select;

// NOTE: use clap for cli args
// const PERIPHERAL_NAME_MATCH_FILTER: &str = "Neuro";
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short='n', long = "name")] // , default_value_t = None)]
    name: Option<String>,
    #[arg(short='f', long = "nus-filter", default_value_t = false)]
    nus_filter: bool
}

// NOTE: BLE UUIDs for NUS copied from bleak example, uart_service.py
// UART_SERVICE_UUID = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E"
// UART_RX_CHAR_UUID = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E"
// UART_TX_CHAR_UUID = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E"
// const UART_SERVICE_UUID: Uuid = Uuid::from_u128(0x6E400001_B5A3_F393_E0A9_E50E24DCCA9E);
const UART_RX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400002_B5A3_F393_E0A9_E50E24DCCA9E);
const UART_TX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400003_B5A3_F393_E0A9_E50E24DCCA9E);

fn is_exit_string(s: &String) -> bool {
    if s.starts_with("exit") {
        return true;
    }
    return false;
}

async fn connect_periph(adapter: &Adapter) -> Result<String, anyhow::Error> {
    // INFO: keep scanning until we find our peripheral
    loop {
        // for adapter in adapter_list.iter() {
        println!("Starting scan...");

        // let nus_scan_filter = ScanFilter { services: vec![UART_SERVICE_UUID] };
        let nus_scan_filter = ScanFilter { services: vec![] };
        adapter
            .start_scan(nus_scan_filter.clone())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(5)).await;
        let peripherals = adapter.peripherals().await?;
        let mut pstrings: Vec<String> = peripherals
            .iter()
            .map(|p| { p.to_string() })
            .collect();
        pstrings.insert(0, String::from("NOT IN LIST; KEEP SCANNING"));
        if let Ok(pdesc) = Select::new("Please choose a BLE peripheral", pstrings.clone()).prompt() {
            if pdesc.starts_with("NOT IN LIST;") {
                continue;
            }

            // getting here means a peripheral has been selected, get index by string
            let index = pstrings.iter().position(|s| pdesc.eq(s)).unwrap();
            let periph= &peripherals[index-1]; // NOTE: minute-one is b/c "NOT IN LIST" above
            if let Err(err) = periph.connect().await {
                eprintln!("Error connecting to peripheral: {}", err);
                continue;
            }

            // let platform_periph: PlatformPeripheral = periph

            // NOTE: after successful connection, return a description string to caller
            // let tmp = periph.deref();
            return Ok(pdesc);
        }
    }
}

fn print_nus_failure() {
    println!("ERROR: Unable to properly configure the BLE characteristics required to use NUS");
    println!("ERROR: NOTE: NUS_TX (BLE notifs from periph) = {UART_TX_CHAR_UUID}");
    println!("ERROR: NOTE: NUS_RX (BLE write to periph) = {UART_RX_CHAR_UUID}");
}

async fn disconnect_periph(p: &PlatformPeripheral) {
    print!("Disconnecting from {:?}... ", p.to_string());
    match p.disconnect().await {
        Ok(_good) => {},
        Err(_bad) => {/* TODO: handle error */ }
    }
    println!("[DONE]");
}

fn press_enter(prompt: &str) {
    println!("{prompt}");
    let mut _input = String::new();
    match(std::io::stdin().read_line(&mut _input)) {
        Ok(_good) => {},
        Err(_bad) => {}
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // NOTE: init logger?
    pretty_env_logger::init();

    // NOTE: parse args
    let args = Args::parse();
    // print args
    println!("args = {args:?}");


    // NOTE: init btleplug
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    println!("Found {} BLE adapter(s)", adapter_list.len());
    println!("Setting up BLE adapter...");
    let adapter = &adapter_list[0];
    if let Ok(adapter_info) = adapter.adapter_info().await {
        println!("adapter = {:?}", adapter_info);
    }

    // NOTE: this connects the adapter (i.e. the central) to the peripheral inside the function
    // NOTE: it modifies the state of adapter
    let pdesc= connect_periph(&adapter).await?;
    println!("Connected to {:?}", pdesc);

    // get access to what should be the only connected peripheral
    let plist = adapter.peripherals().await.unwrap();
    let mut periph: &PlatformPeripheral = &plist[0];
    for pix in 0..plist.len() {
        let pchk = &plist[pix];
        let bchk = pchk.is_connected().await.unwrap();
        if bchk {
            periph = pchk;
            break;
        }
    }
    println!("Connected to {periph:?}");

    periph.discover_services().await?;
    println!("Discovered services...");

    let chars = periph.characteristics();
    println!("Obtained chars = {chars:?}");

    println!("Connected, configuring NUS chars + notifications...");
    let mut nus_send: &Characteristic = &chars.first().unwrap();
    let mut subscribed_tx = false;
    let mut found_rx: bool = false;
    for c in chars.iter() {
        match c.uuid {
            UART_TX_CHAR_UUID  => {
                if c.properties.contains(CharPropFlags::NOTIFY) {
                    println!("Subscribing to characteristic {:?}", c);
                    if let Ok(_good) = periph.subscribe(c).await {
                        subscribed_tx = true;
                    }
                }
            },
            UART_RX_CHAR_UUID  => {
                println!("Setting nus_send to characteristic {:?}", c);
                found_rx = true;
                nus_send = c;
            }
            _ => ()
        }
    }

    if !(subscribed_tx && found_rx) {
        print_nus_failure();
        disconnect_periph(&periph).await;
        press_enter("Press <ENTER> to exit");
        // TODO: document different possible error codes
        std::process::exit(42000);
    }


    println!("Spawning tokio task as handler for notifications");
    let mut notif_stream = periph.notifications().await?;
    // TODO: determine if we need to cleanly stop this task
    let notifs_handler = tokio::spawn(async move {
        loop {
            if let Some(data) = notif_stream.next().await {
                // println!(
                //     "Received data from NUS-TX [{:?}]: {:?}",
                //     data.uuid, data.value
                // );
                let s = match String::from_utf8(data.value) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                print!("{s}");
            }
        }
    });

    // NOTE: init reedline
    let mut line_editor = Reedline::create();
    // let prompt = DefaultPrompt::default();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::CurrentDateTime,
        right_prompt: DefaultPromptSegment::Basic(periph.to_string())
    };

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                if is_exit_string(&buffer) {
                    break;
                }
                // NOTE: add newline char
                let tmp_s: String = format!("{buffer}\n");
                let tmp_bytes = tmp_s.as_bytes();
                // println!("sending -->{:?}<--", buffer);
                let wr_result = periph.write(nus_send, tmp_bytes, WriteType::WithoutResponse).await;
                match wr_result {
                    Ok(_good) => {
                        // println!("Success = {good:?}");
                    },
                    Err(bad) => {
                        println!("Error writing to {nus_send:?} = {bad:?}");
                        /* TODO - handle error */
                    }
                }
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                break;
            }
            x => {
                println!("Event: {:?}", x);
            }
        }
    }
    println!("Stopping tokio task handler (notifications)...");
    notifs_handler.abort();
    disconnect_periph(&periph).await;
    press_enter("Press <ENTER> to exit");

    Ok(())
}


