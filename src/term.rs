
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
use tokio::{select, time::interval};
use tracing::info;

// use anyhow::Error;
// use tokio::io::Error;
// use std::io::Error;

pub async fn term_run(transport: &impl NusTransport) {
    let prompt = {
        // let prompt_seg_1 = "╭>╮".magenta().on_dark_grey().to_string();
        let prompt_seg_1 = ">".magenta().on_dark_grey().to_string();
        let prompt_seg_2 = " ".to_string();
        format!("{}{}", prompt_seg_1, prompt_seg_2)
        // "> ".to_string()
    };

    let maybe_1= TerminalAsync::try_new(prompt.as_str()).await;
    let maybe_2 = maybe_1.expect("Failed to get maybe2");
    let maybe_3 = maybe_2.expect("Failed to get maybe3");

    let mut ta = maybe_3;


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


    // Initialize tracing w/ the "async stdout" (SharedWriter), and file writer.
    // TracingConfig::new_file_and_display(
    //     None,
    //     DisplayPreference::SharedWriter::
    //     // (terminal_async.clone_shared_writer())
    // )
    // .install_global()?;
    // let shwr = terminal_async.clone_shared_writer();
    // let dp = DisplayPreference::SharedWriter(shwr);
    // TracingConfig::new_file_and_display(
    //     None, 
    //     dp
    // );

    // TODO: start bg task to print
    // let mut state = State::default();
    // let mut interval_1_task = interval(state.task_1_state.interval_delay);

    // terminal_async.println(get_info_message().to_string()).await;
    // let mut line_editor = DefaultEditor::new().expect("Can't make line_editor");
    // println!("");
    info!("starting term loop");

    // INFO: rustyline loop
    loop {
        // _ = interval_1_task.tick() => {
        //     task_1::tick(&mut state, &mut terminal_async.clone_shared_writer())?;
        // },
        // _ = interval_2_task.tick() => {
        //     task_2::tick(&mut state, &mut terminal_async.clone_shared_writer())?;
        // },

        let result_readline_event = ta.get_readline_event().await;
        match result_readline_event {
            Ok(readline_event) => {
                match readline_event {
                    // User input event.
                    ReadlineEvent::Line(user_input) => {
                        // let mut_state = &mut state;
                        let shwr= &mut ta.clone_shared_writer();
                        let readline = &mut ta.readline;
                        match writeln!(shwr, "[SENT '{}']", user_input.magenta().on_dark_grey()) {
                            Ok(_good) => (),
                            Err(_bad) => println!("hmmm... {:?}", _bad),
                        };
                        
                        // let control_flow = process_input_event::process(
                        //     user_input, mut_state, shared_writer, readline)?;
                        // if let ControlFlow::Break(_) = control_flow {
                        //     break;
                        // }
                    }
                    // Resize event.
                    ReadlineEvent::Resized => {
                        let shwr= &mut ta.clone_shared_writer();
                        match writeln!(shwr, "{}", "Terminal resized!".yellow()).into_diagnostic() {
                            Ok(_good) => (), //println!("yay!"),
                            Err(_bad) => println!("hmmm... {:?}", _bad),
                        };
                    }
                    // Ctrl+D, Ctrl+C.
                    ReadlineEvent::Eof | ReadlineEvent::Interrupted => {
                        break;
                    }
                }
            },
            Err(err) => {
                let msg_1 = format!("Received err: {}", format!("{:?}",err).red());
                let msg_2 = format!("{}", "Exiting...".red());
                // terminal_async.println(msg_1).await;
                // terminal_async.println(msg_2).await;
                break;
            },
        } // match result_readline_event

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


// Run terminal
// pub async fn term_run(transport: &impl NusTransport) -> Result<(), Box<dyn Error>> {
//     println!("TODO... run terminal...");
//     Ok(())
// }
