
use crate::transport::NusTransport;

use std::{fs,
    io::{stderr, Write},
    ops::ControlFlow,
    path::{self, PathBuf},
    str::FromStr as _,
    sync::Arc,
    time::Duration};

use crossterm::style::Stylize as _;
use miette::{miette, Error, IntoDiagnostic as _};
use r3bl_core::{tracing_logging::tracing_config::TracingConfig,
          DisplayPreference,
          SendRawTerminal,
          SharedWriter,
          StdMutex};
use r3bl_terminal_async::{Readline,
                    ReadlineEvent,
                    Spinner,
                    SpinnerStyle,
                    TerminalAsync};

                    use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use tokio::{select, time::{error::Elapsed, interval, timeout_at}};
use tracing::info;

// use anyhow::Error;
// use tokio::io::Error;
// use std::io::Error;

use tokio::time::timeout;
pub async fn term_run(transport: &impl NusTransport) {
    let prompt = {
        let prompt_seg_1 = "NUS>".magenta().on_dark_grey().to_string();
        let prompt_seg_2 = " ".to_string();
        format!("{}{}", prompt_seg_1, prompt_seg_2)
    };

    let maybe_1= TerminalAsync::try_new(prompt.as_str()).await;
    let maybe_2 = maybe_1.expect("Failed to get maybe2");
    let maybe_3 = maybe_2.expect("Failed to get maybe3");
    let mut ta = maybe_3;

    // start task to print returned bytes as utf8



    // Initialize tracing w/ the "async stdout" (SharedWriter), and file writer.
    // TracingConfig::new_file_and_display(
    //         None,
    //         DisplayPreference::SharedWriter(terminal_async.clone_shared_writer()),
    // )
    // .install_global()?;

    // TODO: take this as an input for completions
    // Pre-populate the readline's history with some entries.
    // for command in Command::iter() {
    //     terminal_async
    //         .readline
    //         .add_history_entry(command.to_string());
    // }

    // TODO: start bg task to print
    // let mut state = State::default();
    // let mut interval_1_task = interval(state.task_1_state.interval_delay);

    // terminal_async.println(get_info_message().to_string()).await;
    // let mut line_editor = DefaultEditor::new().expect("Can't make line_editor");
    // println!("");
    info!("starting term loop");

    // debug!("Spawning tokio task as handler for notifications");
    // let mut notif_stream = periph.notifications().await.unwrap();
    // let (mut ble_notif_recv, mut ble_notif_send) = tokio::sync::mpsc::channel::<String>(8);
    // let mut notif_recv_chan = tokio::sync::mpsc::channel()
    // let notifs_handler = tokio::spawn(async move {
    //     let mut notif_count = 0;
    //     let shwr = ta.clone_shared_writer();
    //     loop {
    //         let res= transport.recv().await;
    //         match res {
    //             Ok(vu8) => {
    //                 let res2 = 
    //                 writeln!(
    //                     shwr, "{}",
    //                     String::from_utf8(vu8)
    //                         .expect("Unable to create utf8-str from received bytes")
    //                 );
    //                 match res2 {
    //                     Ok(_good2) => {
    //                         notif_count += 1;
    //                     },
    //                     Err(_bad2) => (),
    //                 }
    //             },
    //             Err(_bad) => {
    //             }
    //         }
    //     }
    // });
    // let v = data.value;
    // match String::from_utf8(v.clone()) {
    //     Ok(s) => {
    //     },
    //     Err(_bad) => {
    //     }
    // };


    // let shwr_for_notifs = ta.clone_shared_writer();
    // // let mut notif_stream = periph.notifications().await.unwrap();
    // // let (mut ble_notif_recv, mut ble_notif_send) = tokio::sync::mpsc::channel::<String>(8);
    // // let mut notif_recv_chan = tokio::sync::mpsc::channel()
    // let notifs_handler = tokio::spawn(async move {
    //     let mut notif_count = 0;
    //     let shwr = ta.clone_shared_writer();
    //     loop {
    //         let res= transport.recv().await;
    //         match res {
    //             Ok(vu8) => {
    //                 let res2 = 
    //                 writeln!(
    //                     shwr, "{}",
    //                     String::from_utf8(vu8)
    //                         .expect("Unable to create utf8-str from received bytes")
    //                 );
    //                 match res2 {
    //                     Ok(_good2) => {
    //                         notif_count += 1;
    //                     },
    //                     Err(_bad2) => (),
    //                 }
    //             },
    //             Err(_bad) => {
    //             }
    //         }
    //     }
    // });
 
    loop {
        let res_rl_evt = timeout(
            Duration::from_millis(250),
            ta.get_readline_event()
        );
        match res_rl_evt.await {
            // INFO: Line
            Ok(Ok(ReadlineEvent::Line(tmps))) => {
                let tmps_newline = format!("{}\n", tmps);
                let shwr= &mut ta.clone_shared_writer();
                let tmpb = tmps_newline.as_bytes();
                match transport.send(tmpb).await {
                    Ok(_good) => {
                        match writeln!(shwr, "[SENT '{}']", tmps.magenta().on_dark_grey()) {
                            Ok(_good2) => (),
                            Err(_bad2) => println!("hmmm... {}", _bad2),
                        };
                    },
                    Err(_bad) => {
                        println!("hmmm... {}", _bad);
                    }
                }
            }

            // INFO: Resized
            Ok(Ok(ReadlineEvent::Resized)) => {
                let shwr= &mut ta.clone_shared_writer();
                match writeln!(shwr, "{}", "Terminal resized!".yellow()).into_diagnostic() {
                Ok(_good) => (), //println!("yay!"),
                Err(_bad) => println!("hmmm... {:?}", _bad),
                };
            },

            // INFO: Ctrl+D, Ctrl+C.
            Ok(Ok(ReadlineEvent::Eof | ReadlineEvent::Interrupted)) => {
                    break;
            },
            Ok(Err(rl_err)) => {
                let msg_1 = format!("Received err: {}", format!("{:?}",rl_err).red());
                let msg_2 = format!("{}", "Exiting...".red());
                break;
            },
            Err(elapsed) => {
                // if we timeout, just try to recv + print immediately
                let recv_chk = timeout(
                    Duration::from_millis(20),
                    transport.recv()
                );
                match recv_chk.await {
                    Ok(Ok(vu8)) => {
                        let tmps = String::from_utf8(vu8.clone())
                            .expect(format!("Can't parse utf8 string from {:?}", vu8).as_str());
                        // let tmps = format!("{}\n", tmps);
                        let shwr= &mut ta.clone_shared_writer();
                        // let tmpb = tmps.as_bytes();
                        // match transport.send(tmpb).await {
                        match write!(shwr, "{}", tmps.white().on_dark_blue()) {
                            Ok(_good2) => (),
                            Err(_bad2) => println!("hmmm... {}", _bad2),
    
                        }
                    },
                    Ok(Err(boxd_err)) => {
                        let msg_1 = format!("Received err: {}", format!("{:?}",boxd_err).red());
                        let msg_2 = format!("{}", "Exiting...".red());
                    }
                    Err(elapsed) => {
                        // this is expected, nothing to do
                    }
                } // end: match recv_chk.await

            } // end: match result_readline_event

        } // end: match res_rl_evt.await

    } // end loop


    info!("nusterm is exiting...");
    
    // NOTE: disconnect periph issues its own print/info statements
    // disconnect_periph(&periph).await;
    
    // info!("Stopping tokio task handler (notifications)... ");
    // notifs_handler.abort();
    // info!("[DONE]");
    
    // TODO: put helpful info in this 'exit message'
    // press_enter("All done\nPress <ENTER> to exit");
    
    // Ok(())

}

