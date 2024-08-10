// Create a default reedline object to handle user input

use anyhow;

use clap::{ArgAction, Parser, Subcommand};

use reedline::{DefaultPrompt, Reedline, Signal};


use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
// use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
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
const UART_SERVICE_UUID: Uuid = Uuid::from_u128(0x6E400001_B5A3_F393_E0A9_E50E24DCCA9E);
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
        if let Ok(psel) = Select::new("Please choose a BLE peripheral", pstrings.clone()).prompt() {
            if psel.starts_with("NOT IN LIST;") {
                continue;
            }

            // getting here means a peripheral has been selected, get index by string
            let index = pstrings.iter().position(|s| psel.eq(s)).unwrap();
            let peripheral = &peripherals[index-1]; // NOTE: minute-one is b/c "NOT IN LIST" above
            if let Err(err) = peripheral.connect().await {
                eprintln!("Error connecting to peripheral: {}", err);
                continue;
            }

            // NOTE: after successful connection, return a description string to caller
            return Ok(psel);
        }
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

    // NOTE: init reedline
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();

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

    // INFO: optionally filter on NUS service UUID, aka UART_SERVICE_UUID
    // let nus_scan_filter = {
    //     if args.nus_filter {
    //         ScanFilter { services: vec![UART_SERVICE_UUID] }
    //     } else {
    //         ScanFilter::default()
    //     }
    // };

    let psel= connect_periph(&adapter).await?;
    println!("Connected to {:?}", psel);

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                println!("We processed: {}", buffer);
                if is_exit_string(&buffer) {
                    println!("\nGoodbye!");
                    break;
                }
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nGoodbye!");
                break;
            }
            x => {
                println!("Event: {:?}", x);
            }
        }
    }
    return Ok(());
}
