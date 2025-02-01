use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::Result;
use simplelog::warn;

use crate::config::LoadFromTomlFile;

pub fn for_each_content(path: &Path) -> Result<Vec<(PathBuf, String, String)>> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| {
            entry
                .ok()
                .filter(|entry| !entry.path().is_dir())
                .and_then(|entry| {
                    let path = entry.path();
                    match (path.file_name(), path.file_stem()) {
                        (Some(name), Some(stem)) => Some((
                            path.to_owned(),
                            name.to_string_lossy().to_string(),
                            stem.to_string_lossy().to_string(),
                        )),
                        _ => {
                            warn!("Failed to read file names: {:?}", path);
                            None
                        }
                    }
                })
        })
        .collect())
}

pub fn for_each_content_toml<T: LoadFromTomlFile>(
    path: &Path,
    error_message: &str,
) -> Result<Vec<(PathBuf, String, String, T)>> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| {
            entry
                .ok()
                .filter(|entry| !entry.path().is_dir())
                .and_then(|entry| match T::from_file(&entry.path()) {
                    Ok(value) => {
                        let path = entry.path();
                        match (path.file_name(), path.file_stem()) {
                            (Some(name), Some(stem)) => Some((
                                path.to_owned(),
                                name.to_string_lossy().to_string(),
                                stem.to_string_lossy().to_string(),
                                value,
                            )),
                            _ => {
                                warn!("Failed to read file names: {:?}", path);
                                None
                            }
                        }
                    }
                    Err(error) => {
                        warn!("{}@{:?}: {:?}", error_message, entry.path(), error);
                        None
                    }
                })
        })
        .collect())
}
