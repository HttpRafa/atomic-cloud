use inquire::{Autocomplete, CustomUserError};
use inquire::autocompletion::Replacement;

#[derive(Clone)]
pub struct SimpleAutoComplete(Vec<String>);

impl SimpleAutoComplete {
    pub fn from_slices(values: Vec<&'static str>) -> Self {
        SimpleAutoComplete(values.iter().map(|string| string.to_string()).collect())
    }
    pub fn from_strings(values: Vec<String>) -> Self {
        SimpleAutoComplete(values)
    }
}

impl Autocomplete for SimpleAutoComplete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(self.0.clone().iter()
            .filter(|s| s.to_lowercase().contains(&input.to_lowercase()))
            .map(|s| s.to_owned())
            .collect())
    }

    fn get_completion(&mut self, _input: &str, _highlighted_suggestion: Option<String>) -> Result<Replacement, CustomUserError> {
        Ok(None)
    }
}