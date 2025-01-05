use loading::Loading;
use start::ConnectionStartMenu;

use crate::{
    application::profile::{Profile, Profiles},
    VERSION,
};

use super::MenuResult;

mod cloudlet;
mod deployment;
mod general;
mod resource;
mod start;
mod unit;
mod user;

pub struct ConnectionMenu;

impl ConnectionMenu {
    pub async fn show(mut profile: Profile, profiles: &mut Profiles) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Connecting to controller \"{}\" at {}",
            profile.name, profile.url
        ));
        match profile.establish_connection().await {
            Ok(mut connection) => {
                if connection.outdated {
                    progress.warn(format!("The controller is running an outdated protocol version {} compared to this client running {}", connection.protocol, VERSION.protocol));
                }
                progress.success("Successfully connected to the controller");
                progress.end();
                ConnectionStartMenu::show(&mut profile, &mut connection, profiles).await
            }
            Err(error) => {
                progress.fail(format!("Failed to connect to the controller: {}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }
}
