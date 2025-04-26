use std::{
    sync::{Arc, Weak},
    vec,
};

use simplelog::{debug, info, warn};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    process::ChildStdin,
    spawn,
    sync::Mutex,
};

use super::STD_BUFFER_SIZE;

pub struct ManagedStdin(Arc<Mutex<BufWriter<ChildStdin>>>);

impl ManagedStdin {
    pub fn new(stdin: ChildStdin) -> Self {
        let writer = Arc::new(Mutex::new(BufWriter::new(stdin)));
        spawn(Self::copy_task(Arc::downgrade(&writer)));

        Self(writer)
    }

    pub async fn write_line(&self, line: T) where T: Into<Cow<'a, str>> {
        let mut writer = self.0.lock().await;
        if let Err(error) = writer.write_all(format!("{line}\n").as_bytes()).await {
            warn!("Failed to write to stdin of child process: {}", error);
        }
        if let Err(error) = writer.flush().await {
            warn!("Failed to flush stdin of child process: {}", error);
        }
        drop(writer);
        info!("-> {}", line);
    }

    async fn copy_task(writer: Weak<Mutex<BufWriter<ChildStdin>>>) {
        debug!("Starting stdin copy task");
        while let Some(writer) = writer.upgrade() {
            let mut buffer = vec![0; STD_BUFFER_SIZE];
            match io::stdin().read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let mut writer = writer.lock().await;
                    if let Err(error) = writer.write_all(&buffer[..n]).await {
                        warn!("Failed to write to stdin of child process: {}", error);
                        break;
                    }
                    if let Err(error) = writer.flush().await {
                        warn!("Failed to flush stdin of child process: {}", error);
                        break;
                    }
                }
                Err(error) => {
                    warn!("Failed to read from stdin: {}", error);
                    break;
                }
            }
        }
        debug!("Stdin copy task finished");
    }
}
