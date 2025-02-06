use std::{fmt::Display, str::FromStr};

use anyhow::{Error, Result};
use inquire::{
    validator::ValueRequiredValidator, Confirm, CustomType, InquireError, MultiSelect, Select, Text,
};

mod connection;
mod create_profile;
mod delete_profile;
mod load_profile;
pub mod start;

pub enum MenuResult {
    Success,
    Aborted,
    Failed(Error),
    Exit,
}

pub struct MenuUtils;

impl MenuUtils {
    pub fn handle_error(error: InquireError) -> MenuResult {
        match error {
            InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                MenuResult::Exit
            }
            _ => MenuResult::Failed(error.into()),
        }
    }

    pub fn text(message: &str, help: &str) -> Result<String, InquireError> {
        Text::new(message)
            .with_validator(ValueRequiredValidator::default())
            .with_help_message(help)
            .prompt()
    }

    pub fn parsed_value<T: FromStr + ToString + Clone>(
        message: &str,
        help: &str,
        error: &str,
    ) -> Result<T, InquireError> {
        CustomType::<T>::new(message)
            .with_error_message(error)
            .with_help_message(help)
            .prompt()
    }

    pub fn confirm(message: &str) -> Result<bool, InquireError> {
        Confirm::new(message)
            .with_help_message("Type y or n")
            .prompt()
    }

    pub fn select<T: Display>(
        message: &str,
        help: &str,
        options: Vec<T>,
    ) -> Result<T, InquireError> {
        Select::new(message, options)
            .with_help_message(help)
            .prompt()
    }

    pub fn select_no_help<T: Display>(message: &str, options: Vec<T>) -> Result<T, InquireError> {
        Select::new(message, options).prompt()
    }

    pub fn multi_select_no_help<T: Display>(
        message: &str,
        options: Vec<T>,
    ) -> Result<Vec<T>, InquireError> {
        MultiSelect::new(message, options).prompt()
    }
}
