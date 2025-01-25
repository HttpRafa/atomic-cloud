use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::{
    io::{BufRead, BufReader, BufWriter, Read},
    marker::PhantomData,
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
    sync::Mutex,
};

use anyhow::{anyhow, Result};

pub struct DriverProcess {
    /* Process */
    process: Child,

    /* Std Readers */
    stdout: ProcessStream<ChildStdout>,
    stderr: ProcessStream<ChildStderr>,

    /* StdIn Writer */
    stdin: BufWriter<ChildStdin>,
}

impl DriverProcess {
    pub fn new(mut process: Child, direct: bool) -> Result<Self> {
        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .ok_or(anyhow!("Failed to take stdout from child process"))?,
        );
        let stderr = BufReader::new(
            process
                .stderr
                .take()
                .ok_or(anyhow!("Failed to take stderr from child process"))?,
        );
        let stdin = BufWriter::new(
            process
                .stdin
                .take()
                .ok_or(anyhow!("Failed to take stdin from child process"))?,
        );

        let (stdout, stderr) = if direct {
            (ProcessStream::Direct(stdout), ProcessStream::Direct(stderr))
        } else {
            (
                ProcessStream::Async(AsyncBufReader::new(stdout)),
                ProcessStream::Async(AsyncBufReader::new(stderr)),
            )
        };

        Ok(Self {
            process,
            stdout,
            stderr,
            stdin,
        })
    }

    pub fn get_process(&mut self) -> &mut Child {
        &mut self.process
    }

    pub fn get_stdout(&mut self) -> &mut ProcessStream<ChildStdout> {
        &mut self.stdout
    }

    pub fn get_stderr(&mut self) -> &mut ProcessStream<ChildStderr> {
        &mut self.stderr
    }

    pub fn get_stdin(&mut self) -> &mut BufWriter<ChildStdin> {
        &mut self.stdin
    }
}

pub enum ProcessStream<T> {
    Direct(BufReader<T>),
    Async(AsyncBufReader<T>),
}

pub struct AsyncBufReader<T> {
    receiver: Mutex<Receiver<String>>,
    phantom: PhantomData<T>,
}

impl<T: Read + Send + 'static> AsyncBufReader<T> {
    pub fn new(mut reader: BufReader<T>) -> Self {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut buffer = String::new();
            while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
                sender.send(buffer.clone()).unwrap();
                buffer.clear();
            }
        });

        AsyncBufReader {
            receiver: Mutex::new(receiver),
            phantom: PhantomData,
        }
    }

    pub fn try_recv(&self) -> Option<String> {
        self.receiver
            .lock()
            .expect("Failed to lock reader for readline")
            .try_recv()
            .ok()
    }
}
