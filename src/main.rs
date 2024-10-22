// Create a default reedline object to handle user input

mod transport;
mod transport_btle;
mod transport_mock;
mod term;

use transport::nus_uuids::*;


use std::error::Error;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};
use std::result::Result;
use std::time::Duration;

// use text_io::read;
use anyhow;
use chrono::{Datelike, Local};

use clap::Parser;
// use reedline::{DefaultPrompt, DefaultPromptSegment, EditCommand, ExternalPrinter, Reedline, Signal};
// use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, ExternalPrinter};

use btleplug::api::{
    BDAddr, Central, CharPropFlags, Characteristic, Manager as _,
    Peripheral as ApiPeripheral, PeripheralProperties,
    ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral as PlatformPeripheral};
use futures::stream::StreamExt;

// use tokio::sync::mpsc;
use tokio;
use tokio::time;
use transport_mock::NusTransportMock;
use uuid::Uuid;

use inquire::Select;

use log::{debug, info, warn, error};
use simplelog::{CombinedLogger, ColorChoice, Config, ConfigBuilder, LevelFilter,
                SimpleLogger, TerminalMode, WriteLogger};

use ansi_term::Colour::{Black, Blue, Yellow, White, Red, RGB};


// NOTE: use clap for cli args
// const PERIPHERAL_NAME_MATCH_FILTER: &str = "Neuro";
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// BLE NUS Target Name
    #[arg(short = 'n', long = "name", default_value = "")] // , default_value_t = None)]
    name: String,

    /// BLE Address
    #[arg(short = 'a', long = "addr", default_value = "")] // , default_value_t = None)]
    addr: String,
}



// fn is_exit_string(s: &String) -> bool {
//     if s.starts_with("exit") {
//         return true;
//     }
//     return false;
// }
//

// this is a pure function; output is a strict function of input
// fn periph_desc_string(props: &PeripheralProperties) -> String {
//     let mut dlist: Vec<String> = vec![]; // desc list
//     // name first
//     if let Some(name) = &props.local_name {
//         dlist.push(format!("name={}", name));
//     }
//     // rssi next
//     if let Some(rssi_val) = &props.rssi {
//         dlist.push(format!("rssi={}", rssi_val));
//     }
//     // addr next
//     dlist.push(format!("addr={}", props.address));
//     // return a joined version as the output
//     dlist.join(" : ")
// }


// async fn connect_periph(adapter: &Adapter) -> Result<String> {
//     // INFO: keep scanning until we find our peripheral
//     loop {
//         // for adapter in adapter_list.iter() {
//         info!("Starting scan...");

//         // let nus_scan_filter = ScanFilter { services: vec![UART_SERVICE_UUID] };
//         let nus_scan_filter = ScanFilter { services: vec![] };
//         adapter
//             .start_scan(nus_scan_filter.clone())
//             .await
//             .expect("Can't scan BLE adapter for connected devices...");
//         time::sleep(Duration::from_secs(5)).await;
//         let peripherals = adapter.peripherals().await.unwrap();
//         let mut pstrings: Vec<String> = vec![String::from("NOT IN LIST; KEEP SCANNING")];
//         for p in &peripherals {
//             pstrings.push(match p.properties().await {
//                 Ok(Some(props)) => {
//                     periph_desc_string(&props)
//                 },
//                 _ => {
//                     format!("ERR: unable to fetch properties")
//                 }
//             });
//         }

//         if let Ok(pdesc) = Select::new("Please choose a BLE peripheral", pstrings.clone()).prompt()
//         {
//             if pdesc.starts_with("NOT IN LIST;") {
//                 continue;
//             }

//             // getting here means a peripheral has been selected, get index by string
//             let index = pstrings.iter().position(|s| pdesc.eq(s)).unwrap();
//             let periph = &peripherals[index - 1]; // NOTE: minute-one is b/c "NOT IN LIST" above
//             if let Err(err) = periph.connect().await {
//                 error!("Error connecting to peripheral: {}", err);
//                 continue;
//             }

//             // NOTE: after successful connection, return a description string to caller
//             // let tmp = periph.deref();
//             return Ok(pdesc);
//         }
//     }
// }

// fn print_nus_failure() {
//     error!("Unable to properly configure the BLE characteristics required to use NUS");
//     error!("NOTE: NUS_TX (BLE notifs from periph) = {NUS_TX_CHAR_UUID}");
//     error!("NOTE: NUS_RX (BLE write to periph) = {NUS_RX_CHAR_UUID}");
// }

// async fn disconnect_periph(p: &PlatformPeripheral) {
//     let addr = p.address();
//     debug!("Disconnecting from {:?}... ", addr);
//     match p.disconnect().await {
//         Ok(_good) => {}
//         Err(_bad) => { /* TODO: handle error */ }
//     }
//     debug!("[DONE]");
// }

fn press_enter(prompt: &str) {
    println!("{prompt}");
    let mut _input = String::new();
    match std::io::stdin().read_line(&mut _input) {
        Ok(_good) => {}
        Err(_bad) => {}
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // NOTE: set up logger
    let dt = Local::now();
    let mut logs_dir_path = PathBuf::new();
    logs_dir_path.push(".");
    logs_dir_path.push("LOGS");
    logs_dir_path.push(dt.format("%y_%m_%b").to_string());
    match create_dir_all(logs_dir_path.clone()) {
        Ok(_good) => {},
        Err(_bad) => {
            // FIXME: handle error
        },
    }

    let mut line_editor = DefaultEditor::new().expect("Can't make line_editor");
    // let mut ep1 = line_editor.create_external_printer().expect("Can't make ext. printer");
    // let mut ep2 = line_editor.create_external_printer().expect("Can't make ext. printer");
 
    // NOTE: parse args
    let args = Args::parse();
    // print args
    info!("args = {args:?}");

    println!("starting @ {dt:?}");
    // let log_file_name= Path::new(dt.format("nusterm_%y-%m-%d_%H_%M_%S.log"));
    let log_start_str = dt.format("%y-%m-%d_%H_%M_%S").to_string();

    let code_log_fname = format!("nusterm_{}.log", log_start_str);
    let code_log_fpath = logs_dir_path.join(code_log_fname);

    let log_config = ConfigBuilder::new()
        .set_time_format_rfc2822()
        .add_filter_ignore(format!("{}","rustyline"))
        // .set_time_offset_to_local().unwrap()
        .build();
    CombinedLogger::init(vec![
        // #[cfg(feature = "termcolor")]
        // TermLogger::new(
        //     LevelFilter::Info,
        //     log_config.clone(),
        //     TerminalMode::Mixed,
        //     ColorChoice::Auto,
        // ),
        //
        // #[cfg(not(feature = "termcolor"))]
        // SimpleLogger::new(LevelFilter::Info, log_config.clone()),

        WriteLogger::new(
            LevelFilter::Debug,
            log_config.clone(),
            File::create(code_log_fpath).unwrap(),
        ),
    ]).unwrap();


    // let transport = &transport_btleplug::NusTransportBtleplug::new(args.name, addr).await?;

    // INFO: mock transport
    // let tmpaddr:std::result::Result<BDAddr, btleplug::api::ParseBDAddrError> = BDAddr::from_str_delim("");
    // let transport = &NusTransportMock::new(String::from("mock"));
    
    // INFO: btleplug transport
    let addr = BDAddr::from_str_delim(&args.addr);
    let transport = 
        &transport_btle::NusTransportBtleplug::new(args.name, addr)
        .await
        .expect("Unable to create NUS Transport");

    let _term_result = term::term_run(transport).await;

    // let _dc_result = transport.disconnect().await;



    // NOTE: init btleplug
    // let manager = Manager::new().await.unwrap();
    // let adapter_list = manager.adapters().await.unwrap();
    // if adapter_list.is_empty() {
    //     error!("No Bluetooth adapters found");
    // }

    // info!("Found {} BLE adapter(s)", adapter_list.len());
    // info!("Setting up BLE adapter...");
    // let adapter = &adapter_list[0];
    // if let Ok(adapter_info) = adapter.adapter_info().await {
    //     info!("adapter = {:?}", adapter_info);
    // }


    // NOTE: this connects the adapter (i.e. the central) to the peripheral inside the function
    // NOTE: it modifies the state of adapter
    // let pdesc = connect_periph(&adapter).await?;

    // get access to what should be the only connected peripheral
    // let plist = adapter.peripherals().await.unwrap();
    // let mut periph: &PlatformPeripheral = &plist[0];
    // for pix in 0..plist.len() {
    //     let pchk = &plist[pix];
    //     let bchk = pchk.is_connected().await.unwrap();
    //     if bchk {
    //         periph = pchk;
    //         break;
    //     }
    // }
    // info!("Connected to {:?}", pdesc);
    // debug!("Connected to {periph:?}");

    // info!("Discovering services...");
    // periph.discover_services().await.unwrap();

    info!("nusterm is exiting...");
    
    // NOTE: disconnect periph issues its own print/info statements
    // disconnect_periph(&periph).await;
    
    // info!("Stopping tokio task handler (notifications)... ");
    // notifs_handler.abort();
    // info!("[DONE]");
    
    // TODO: put helpful info in this 'exit message'
    // press_enter("All done\nPress <ENTER> to exit");
    
    // #[cfg(feature = "with-file-history")]
    // line_editor.save_history("history.txt");

    Ok(())

}
