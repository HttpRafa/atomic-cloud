use std::{thread, time::Duration};

use inquire::Confirm;
use loading::Loading;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct RequestStopMenu;

impl RequestStopMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        match Confirm::new("Do you really want to stop this controller?")
            .with_help_message("This will stop all servers and kick all users | Type y or n")
            .prompt()
        {
            Ok(true) => {
                let progress = Loading::default();
                progress.text(format!("Stopping controller \"{}\"", profile.name));
                match connection.client.request_stop().await {
                    Ok(_) => {
                        thread::sleep(Duration::from_secs(3));
                        progress.success("Controller stopped 👍");
                        progress.end();
                        MenuResult::Exit
                    }
                    Err(err) => {
                        progress.fail(format!(
                            "Ops. Something went wrong while stopping the controller | {}",
                            err
                        ));
                        progress.end();
                        MenuResult::Failed
                    }
                }
            }
            Ok(false) | Err(_) => MenuResult::Aborted,
        }
    }
}
