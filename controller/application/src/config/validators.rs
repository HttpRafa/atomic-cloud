use std::net::IpAddr;
use inquire::CustomUserError;
use inquire::validator::{StringValidator, Validation};

#[derive(Clone)]
pub struct PortValidator;

impl PortValidator {
    pub fn new() -> Self {
        PortValidator {}
    }
}

impl StringValidator for PortValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let result = input.parse::<u16>();
        if result.is_ok() {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(result.unwrap_err().into()))
        }
    }
}

#[derive(Clone)]
pub struct UnsignedValidator;

impl UnsignedValidator {
    pub fn new() -> Self {
        UnsignedValidator {}
    }
}

impl StringValidator for UnsignedValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let result = input.parse::<u32>();
        if result.is_ok() {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(result.unwrap_err().into()))
        }
    }
}

#[derive(Clone)]
pub struct AddressValidator;

impl AddressValidator {
    pub fn new() -> Self {
        AddressValidator {}
    }
}

impl StringValidator for AddressValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let result = input.parse::<IpAddr>();
        if result.is_ok() {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(result.unwrap_err().into()))
        }
    }
}