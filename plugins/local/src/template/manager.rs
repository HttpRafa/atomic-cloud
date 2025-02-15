use std::{collections::HashMap, fs};

use anyhow::Result;
use stored::StoredTemplate;

use crate::{info, storage::Storage};

use super::Template;

pub struct TemplateManager {
    templates: HashMap<String, Template>,
}

impl TemplateManager {
    pub async fn init() -> Result<Self> {
        info!("Loading templates...");
        let mut templates = HashMap::new();

        let directory = Storage::templates_directory(false);
        if !directory.exists() {
            fs::create_dir_all(&directory)?;
        }

        for (_, _, name, value) in Storage::for_each_content_toml::<StoredTemplate>(&directory, "Failed to read template from file").await? {
            info!("Loading template {}", name);
        }

        info!("Loaded {} template(s)", templates.len());
        Ok(Self { templates })
    }
}

pub(super) mod stored {
    use std::{collections::HashMap, path::PathBuf};

    use common::file::SyncLoadFromTomlFile;
    use getset::Getters;
    use serde::{Deserialize};

    use crate::template::PlatformScript;

    #[derive(Deserialize, Getters)]
    pub struct StoredTemplate {
                /* Template */
                #[getset(get = "pub")]
                description: String,
                #[getset(get = "pub")]
                version: String,
                #[getset(get = "pub")]
                authors: Vec<String>,
        
                /* Files */
                #[getset(get = "pub")]
                exclusions: Vec<PathBuf>,
        
                /* Environment */
                #[getset(get = "pub")]
                environment: HashMap<String, String>,
        
                /* Commands */
                #[getset(get = "pub")]
                shutdown: Option<String>,
        
                /* Scripts */
                #[getset(get = "pub")]
                prepare: PlatformScript,
                #[getset(get = "pub")]
                startup: PlatformScript,
    }    

    impl SyncLoadFromTomlFile for StoredTemplate {}
    
}