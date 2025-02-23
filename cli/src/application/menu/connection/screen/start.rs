use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::spawn,
};

use anyhow::{anyhow, Result};
use common::error::FancyError;
use futures::executor::block_on;
use rustyline::{error::ReadlineError, DefaultEditor, ExternalPrinter};
use tokio::{select, sync::mpsc};
use tonic::Streaming;

use crate::application::{
    menu::MenuResult,
    network::{
        proto::manage::screen::{Lines, WriteReq},
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

const WRITE_BUFFER_SIZE: usize = 32;

pub struct ScreenMenu;

impl ScreenMenu {
    pub async fn show(
        _profile: &mut Profile,
        connection: &mut EstablishedConnection,
        profiles: &mut Profiles,
        name: &str,
        id: &str,
        stream: Streaming<Lines>,
    ) -> MenuResult {
        match Self::show_internal(connection, profiles, name, id, stream).await {
            Ok(result) => result,
            Err(error) => MenuResult::Failed(error),
        }
    }

    async fn show_internal(
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
        name: &str,
        id: &str,
        mut stream: Streaming<Lines>,
    ) -> Result<MenuResult> {
        let mut editor = DefaultEditor::new()?;
        let mut printer = editor.create_external_printer()?;

        let (writer, mut writer_recv) = mpsc::channel(WRITE_BUFFER_SIZE);
        let signal = Arc::new(AtomicBool::new(true));
        {
            let prompt = format!("| {name} » ");
            let signal = signal.clone();
            spawn(move || {
                while signal.load(Ordering::Relaxed) {
                    match editor.readline(&prompt) {
                        Ok(line) => {
                            if block_on(writer.send(line)).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            if let ReadlineError::Interrupted = error {
                                break;
                            }
                            FancyError::print_fancy(&error.into(), false);
                            break;
                        }
                    }
                }
            });
        }
        loop {
            select! {
                line = writer_recv.recv() => if let Some(line) = line {
                    connection.client.write_to_screen(WriteReq {
                        id: id.to_owned(),
                        data: format!("{line}\n").as_bytes().to_vec(),
                    }).await?;
                } else { break },
                lines = stream.message() => match lines {
                    Ok(Some(lines)) => for line in lines.lines {
                        printer.print(line)?;
                    },
                    Ok(None) => break,
                    Err(error) => return Err(anyhow!(error)),
                }
            }
        }
        signal.store(false, Ordering::Relaxed);

        Ok(MenuResult::Success)
    }
}
