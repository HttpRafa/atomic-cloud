use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use inquire::{validator::ValueRequiredValidator, Confirm, CustomType, MultiSelect, Select, Text};

mod connection;
mod create_profile;
mod delete_profile;
mod load_profile;
pub mod start;

#[derive(PartialEq)]
pub enum MenuResult {
    Success,
    Aborted,
    Failed,
    Exit,
}

pub struct MenuUtils;

impl MenuUtils {
    pub fn text(message: &str, help: &str) -> Result<String> {
        Text::new(message)
            .with_validator(ValueRequiredValidator::default())
            .with_help_message(help)
            .prompt()
            .map_err(|error| error.into())
    }

    pub fn parsed_value<T: FromStr + ToString + Clone>(
        message: &str,
        help: &str,
        error: &str,
    ) -> Result<T> {
        CustomType::<T>::new(message)
            .with_error_message(error)
            .with_help_message(help)
            .prompt()
            .map_err(|error| error.into())
    }

    pub fn confirm(message: &str) -> Result<bool> {
        Confirm::new(message)
            .with_help_message("Type y or n")
            .prompt()
            .map_err(|error| error.into())
    }

    pub fn select<T: Display>(message: &str, help: &str, options: Vec<T>) -> Result<T> {
        Select::new(message, options)
            .with_help_message(help)
            .prompt()
            .map_err(|error| error.into())
    }

    pub fn select_no_help<T: Display>(message: &str, options: Vec<T>) -> Result<T> {
        Select::new(message, options)
            .prompt()
            .map_err(|error| error.into())
    }

    pub fn multi_select_no_help<T: Display>(message: &str, options: Vec<T>) -> Result<Vec<T>> {
        MultiSelect::new(message, options)
            .prompt()
            .map_err(|error| error.into())
    }
}
