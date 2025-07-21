use anyhow::Result;
use simplelog::debug;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    spawn,
    sync::mpsc::{Receiver, Sender, channel},
};

use super::STD_BUFFER_SIZE;

pub struct ManagedStdout(Receiver<Result<String>>);

impl ManagedStdout {
    pub fn new(stdout: ChildStdout) -> Self {
        let (sender, receiver) = channel(STD_BUFFER_SIZE);
        spawn(Self::read_task(sender, BufReader::new(stdout)));

        Self(receiver)
    }

    pub async fn next_line(&mut self) -> Option<Result<String>> {
        self.0.recv().await
    }

    async fn read_task(sender: Sender<Result<String>>, mut stdout: BufReader<ChildStdout>) {
        debug!("Starting stdout read task");
        loop {
            let mut buffer = String::new();
            match stdout.read_line(&mut buffer).await {
                Ok(0) => break,
                Ok(_) => {
                    if sender.send(Ok(buffer.clone())).await.is_err() {
                        debug!("Failed to send buffer: Channel closed");
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender
                        .send(Err(error.into()))
                        .await
                        .map_err(|_| debug!("Failed to send error: Channel closed"));
                    break;
                }
            }
        }
        debug!("Stdout read task finished");
    }
}
