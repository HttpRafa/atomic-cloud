use std::{thread, time::Duration};

use simplelog::debug;
use wasmtime::{Engine, EngineWeak};

const INCRMENT_EPOCH_INTERVAL: Duration = Duration::from_millis(100);

pub struct EpochInvoker {
    engines: Vec<EngineWeak>,
}

impl EpochInvoker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            engines: vec![],
        }
    }

    pub fn push(&mut self, engine: &Engine) {
        self.engines.push(engine.weak());
    }

    pub fn spawn(mut self) {
        debug!("Starting epoch invoker to increment epoch every {:?}", INCRMENT_EPOCH_INTERVAL);
        thread::spawn(move || loop {
            thread::sleep(INCRMENT_EPOCH_INTERVAL);
            self.engines.retain(|engine| {
                if let Some(engine) = engine.upgrade() {
                    engine.increment_epoch();
                    true
                } else {
                    false
                }
            });
            if self.engines.is_empty() {
                debug!("All engines dropped, stopping epoch invoker");
                break;
            }
        });
    }
}