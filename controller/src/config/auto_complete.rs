use inquire::{Autocomplete, CustomUserError};
use inquire::autocompletion::Replacement;

#[derive(Clone)]
pub struct SimpleAutoComplete(Vec<&'static str>);

impl SimpleAutoComplete {
    pub fn new(values: Vec<&'static str>) -> Self {
        SimpleAutoComplete(values)
    }
}

impl Autocomplete for SimpleAutoComplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(self.0.clone().iter()
            .filter(|s| s.to_lowercase().contains(&input.to_lowercase()))
            .map(|s| String::from(*s))
            .collect())
    }

    fn get_completion(&mut self, _input: &str, _highlighted_suggestion: Option<String>) -> Result<Replacement, CustomUserError> {
        Ok(None)
    }
}