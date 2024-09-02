// Create a default reedline object to handle user input

use std::error::Error;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

// use text_io::read;
use anyhow;
use chrono::{Datelike, Local};

use clap::Parser;
use futures::channel::mpsc::Sender;
// use reedline::{DefaultPrompt, DefaultPromptSegment, EditCommand, ExternalPrinter, Reedline, Signal};
use rustyline::error::ReadlineError;
// use rustyline::{DefaultEditor, Result};

use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _,
    Peripheral as ApiPeripheral, PeripheralProperties,
    ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral as PlatformPeripheral};
use futures::stream::StreamExt;

use tokio::sync::mpsc;
// use tokio::sync::mpsc;
use tokio::time;
use uuid::Uuid;

use inquire::Select;

use log::{debug, info, warn, error};
use simplelog::{CombinedLogger, ColorChoice, Config, ConfigBuilder, LevelFilter,
                SimpleLogger, TerminalMode, WriteLogger};


/// This example is taken from https://raw.githubusercontent.com/fdehau/tui-rs/master/examples/user_input.rs
use ratatui::prelude::*;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
// use std::{error::Error, io};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

enum InputMode {
    Normal,
    Editing,
}

// NOTE: use clap for cli args
// const PERIPHERAL_NAME_MATCH_FILTER: &str = "Neuro";
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'n', long = "name")] // , default_value_t = None)]
    name: Option<String>,
    #[arg(short = 'f', long = "nus-filter", default_value_t = false)]
    nus_filter: bool,
}

/// App holds the state of the application
struct ConnectedApp {
    /// Current value of the input box
    input: Input,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    periph: PlatformPeripheral,
    /// Sink for Linebox
    // linebox_send: mpsc::Sender<String>,
    nus_send: Characteristic,
    /// Sink for Messages
    // msg_recv: mpsc::Receiver<String>,
    nus_recv: Characteristic,
    nus_recv_recv: mpsc::Receiver<String>
}

impl ConnectedApp {
    fn from_periph_send_recv(periph: PlatformPeripheral, nus_send: Characteristic, nus_recv: Characteristic, nus_recv_recv: mpsc::Receiver<String>) -> ConnectedApp {
        ConnectedApp {
            input: Input::default(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            periph,
            nus_send,
            nus_recv,
            nus_recv_recv
        }
    }
    async fn nus_write(&mut self, s: String) {
        let b = s.as_bytes();
        match self.periph.write(&self.nus_send, b, WriteType::WithoutResponse).await {
            Ok(_good) => {/* TODO: ??? */},
            Err(_bad) => {/* FIXME: handle err */},
        }
    }
}

// impl Default for ConnectedApp
//     fn default() -> ConnectedApp
//

// NOTE: BLE UUIDs for NUS copied from bleak example, uart_service.py
// UART_SERVICE_UUID = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E"
// UART_RX_CHAR_UUID = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E"
// UART_TX_CHAR_UUID = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E"
// const UART_SERVICE_UUID: Uuid = Uuid::from_u128(0x6E400001_B5A3_F393_E0A9_E50E24DCCA9E);
const UART_RX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400002_B5A3_F393_E0A9_E50E24DCCA9E);
const UART_TX_CHAR_UUID: Uuid = Uuid::from_u128(0x6E400003_B5A3_F393_E0A9_E50E24DCCA9E);

// fn is_exit_string(s: &String) -> bool {
//     if s.starts_with("exit") {
//         return true;
//     }
//     return false;
// }
//

// this is a pure function; output is a strict function of input
fn periph_desc_string(props: &PeripheralProperties) -> String {
    let mut dlist: Vec<String> = vec![]; // desc list
    // name first
    if let Some(name) = &props.local_name {
        dlist.push(format!("name={}", name));
    }

    // rssi next
    if let Some(rssi_val) = &props.rssi {
        dlist.push(format!("rssi={}", rssi_val));
    }

    // addr next
    dlist.push(format!("addr={}", props.address));

    // return a joined version as the output
    dlist.join(" : ")
}


async fn connect_periph(adapter: &Adapter) -> anyhow::Result<String> {
    // INFO: keep scanning until we find our peripheral
    loop {
        // for adapter in adapter_list.iter() {
        info!("Starting scan...");

        // let nus_scan_filter = ScanFilter { services: vec![UART_SERVICE_UUID] };
        let nus_scan_filter = ScanFilter { services: vec![] };
        adapter
            .start_scan(nus_scan_filter.clone())
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(5)).await;
        let peripherals = adapter.peripherals().await.unwrap();
        let mut pstrings: Vec<String> = vec![String::from("NOT IN LIST; KEEP SCANNING")];
        for p in &peripherals {
            pstrings.push(match p.properties().await {
                Ok(Some(props)) => {
                    periph_desc_string(&props)
                },
                _ => {
                    format!("ERR: unable to fetch properties")
                }
            });
        }

        if let Ok(pdesc) = Select::new("Please choose a BLE peripheral", pstrings.clone()).prompt()
        {
            if pdesc.starts_with("NOT IN LIST;") {
                continue;
            }

            // getting here means a peripheral has been selected, get index by string
            let index = pstrings.iter().position(|s| pdesc.eq(s)).unwrap();
            let periph = &peripherals[index - 1]; // NOTE: minute-one is b/c "NOT IN LIST" above
            if let Err(err) = periph.connect().await {
                error!("Error connecting to peripheral: {}", err);
                continue;
            }

            // NOTE: after successful connection, return a description string to caller
            // let tmp = periph.deref();
            return Ok(pdesc);
        }
    }
}

fn print_nus_failure() {
    error!("Unable to properly configure the BLE characteristics required to use NUS");
    error!("NOTE: NUS_TX (BLE notifs from periph) = {UART_TX_CHAR_UUID}");
    error!("NOTE: NUS_RX (BLE write to periph) = {UART_RX_CHAR_UUID}");
}

async fn disconnect_periph(p: &PlatformPeripheral) {
    let addr = p.address();
    debug!("Disconnecting from {:?}... ", addr);
    match p.disconnect().await {
        Ok(_good) => {}
        Err(_bad) => { /* TODO: handle error */ }
    }
    debug!("[DONE]");
}

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
            // TODO: handle error
        },
    }

    // NOTE: parse args
    let args = Args::parse();
    // print args
    info!("args = {args:?}");

    println!("now = {dt:?}");
    // let log_file_name= Path::new(dt.format("nusterm_%y-%m-%d_%H_%M_%S.log"));
    let log_start_str = dt.format("%y-%m-%d_%H_%M_%S").to_string();

    let code_log_fname = format!("nusterm_{}.log", log_start_str);
    let code_log_fpath = logs_dir_path.join(code_log_fname);

    let log_config = ConfigBuilder::new()
        .set_time_format_rfc2822()
        .set_time_offset_to_local().unwrap()
        .build();
    CombinedLogger::init(vec![
        #[cfg(feature = "termcolor")]
        TermLogger::new(
            LevelFilter::Info,
            log_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),

        #[cfg(not(feature = "termcolor"))]
        SimpleLogger::new(LevelFilter::Info, log_config.clone()),

        WriteLogger::new(
            LevelFilter::Debug,
            log_config.clone(),
            File::create(code_log_fpath).unwrap(),
        ),
    ]).unwrap();


    // NOTE: init btleplug
    let manager = Manager::new().await.unwrap();
    let adapter_list = manager.adapters().await.unwrap();
    if adapter_list.is_empty() {
        error!("No Bluetooth adapters found");
    }

    info!("Found {} BLE adapter(s)", adapter_list.len());
    info!("Setting up BLE adapter...");
    let adapter = &adapter_list[0];
    if let Ok(adapter_info) = adapter.adapter_info().await {
        info!("adapter = {:?}", adapter_info);
    }


    // NOTE: this connects the adapter (i.e. the central) to the peripheral inside the function
    // NOTE: it modifies the state of adapter
    let pdesc = connect_periph(&adapter).await?;

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
    info!("Connected to {:?}", pdesc);
    debug!("Connected to {periph:?}");

    info!("Discovering services...");
    periph.discover_services().await.unwrap();

    info!("Configuring NUS chars + notifications...");
    let chars = periph.characteristics();
    let mut nus_recv: &Characteristic = &chars.first().unwrap();
    let mut nus_send: &Characteristic = &chars.first().unwrap();
    let mut subscribed_tx = false;
    let mut found_rx: bool = false;
    for c in chars.iter() {
        match c.uuid {
            UART_TX_CHAR_UUID => {
                debug!("found NUS_TX (nus_recv) characteristic");
                nus_recv = c;
                if c.properties.contains(CharPropFlags::NOTIFY) {
                    debug!("subscribing to characteristic {:?}", c);
                    if let Ok(_good) = periph.subscribe(c).await {
                        subscribed_tx = true;
                    }
                }
            }
            UART_RX_CHAR_UUID => {
                debug!("found NUS_RX (nus_send) characteristic");
                found_rx = true;
                nus_send = c;
            }
            _ => (),
        }
    }

    // if we didn't set up the NUS chars, then bail and inform user
    if !(subscribed_tx && found_rx) {
        print_nus_failure();
        disconnect_periph(&periph).await;
        press_enter("Press <ENTER> to exit");
        // TODO: document different possible error codes
        std::process::exit(42000);
    }


    // Create external printer
    // let printer: ExternalPrinter<String> = ExternalPrinter::default();
    // let mut printer: ExternalPrinter<String> = ExternalPrinter::new(100);
    // let rxSender = printer.sender();
    let (nusRecvSend, nusRecvRecv) = tokio::sync::mpsc::channel::<String>(8);

    debug!("Spawning tokio task as handler for notifications");
    let mut notif_stream = periph.notifications().await.unwrap();
    let notifs_handler = tokio::spawn(async move {
        let mut notif_count = 0;
        loop {
            if let Some(data) = notif_stream.next().await {
                let v = data.value;
                // NOTE: rust is tricky about ownership, we actually need an extra because:
                //       1. String::from_utf8(v) consumes v
                //       2. Err(_e) consumes vclone
                match String::from_utf8(v.clone()) {
                    Ok(s) => {
                        debug!("{{from_dut: '{s}'}}");
                        match nusRecvSend.send(s).await {
                            Ok(_good) => { /* TODO: ???*/ },
                            Err(_bad) => { /* FIXME: handle err*/ }
                        };
                    },
                    Err(_e) => {
                        warn!("NUS_TX: non-utf-data = {:?}", v.clone());
                        debug!("{{from_dut: '{:?}'}}", v.clone());
                    }
                }

                // NOTE: incr count
                notif_count += 1;
            }
        }
    });

    info!("NUS connection is now active");

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // let (linebox_send, linebox_recv) = mpsc::channel(8);
    // let (msg_send, msg_recv) = mpsc::channel(8);


    // create app and run it
    // let app = ConnectedApp::default();
    let app = ConnectedApp::from_periph_send_recv(
        periph.clone(),
        nus_send.clone(),
        nus_recv.clone(),
        nusRecvRecv
    );
    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    info!("nusterm is exiting...");
    
    // NOTE: disconnect periph issues its own print/info statements
    disconnect_periph(&periph).await;
    
    info!("Stopping tokio task handler (notifications)... ");
    notifs_handler.abort();
    info!("[DONE]");
    
    // TODO: put helpful info in this 'exit message'
    press_enter("All done\nPress <ENTER> to exit");
    
    // #[cfg(feature = "with-file-history")]
    // line_editor.save_history("history.txt");
    Ok(())

}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: ConnectedApp) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;
        // transfer from nus_recv_recv to app.messages
        loop {
            if let Ok(s) = app.nus_recv_recv.try_recv() {
                // do thing
                app.messages.push(s);
                continue
            }
            break;
        }

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if 0 == app.input.value().trim().len() {
                            continue;
                        }
                        // FIXME: support flexible newline string here
                        let app_msg: String = format!("<SEND='{}'>", app.input.value());
                        app.messages.push(app_msg); // app.input.value().into());
                                                    //
                        let ble_msg = format!("{}\n", app.input.value());
                        app.nus_write(ble_msg).await;
                        app.input.reset();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &ConnectedApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let text = Text::from(Line::from(msg)).style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor

    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor_position((
                // Put cursor past the end of the input text
                chunks[1].x
                    + ((app.input.visual_cursor()).max(scroll) - scroll) as u16
                    + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            ))
        }
    }
    
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Line::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[2]);
}
