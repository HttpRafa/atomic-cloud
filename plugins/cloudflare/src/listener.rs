use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use regex::Regex;

use crate::{
    error,
    generated::{
        exports::plugin::system::event::GuestListener,
        plugin::system::{data_types::Server, types::ErrorMessage},
    },
    plugin::{batcher::Batcher, config::Entry, dns::Record},
};

pub struct Listener {
    /* Configuration */
    entries: Vec<(Regex, Entry)>,

    /* Batcher */
    batcher: Rc<RefCell<Batcher>>,
}

impl Listener {
    pub fn new(entries: &[Entry], batcher: Rc<RefCell<Batcher>>) -> Self {
        Self {
            entries: entries
                .iter()
                .filter_map(|entry| match Regex::new(&entry.servers) {
                    Ok(servers) => Some((servers, entry.clone())),
                    Err(error) => {
                        error!(
                            "Failed to compile regex({}) for entry({}): {}",
                            entry.servers, entry.name, error
                        );
                        None
                    }
                })
                .collect(),
            batcher,
        }
    }
}

impl GuestListener for Listener {
    fn server_start(&self, _: Server) -> Result<(), ErrorMessage> {
        unimplemented!()
    }

    fn server_stop(&self, server: Server) -> Result<(), ErrorMessage> {
        for (regex, entry) in &self.entries {
            if regex.is_match(&server.name) {
                self.batcher.borrow_mut().delete(entry.clone(), server.uuid);
                break;
            }
        }
        Ok(())
    }

    fn server_change_ready(&self, server: Server, ready: bool) -> Result<(), ErrorMessage> {
        if !ready {
            // Only run if the server is ready
            return Ok(());
        }

        for (regex, entry) in &self.entries {
            if regex.is_match(&server.name) {
                self.batcher.borrow_mut().create(entry.clone(), server);
                break;
            }
        }
        Ok(())
    }
}
