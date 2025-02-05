use std::{thread, time::Duration};

use inquire::InquireError;
use loading::Loading;

use crate::application::{
    menu::{MenuResult, MenuUtils},
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
        match MenuUtils::confirm("Are you sure you want to stop this controller? This will stop all servers and disconnect all users.")
        {
            Ok(true) => {
                let progress = Loading::default();
                progress.text(format!("Stopping controller \"{}\"", profile.name));
                match connection.client.request_stop().await {
                    Ok(_) => {
                        thread::sleep(Duration::from_secs(3));
                        progress.success("Controller stopped successfully 👍");
                        progress.end();
                        MenuResult::Exit
                    }
                    Err(error) => {
                        progress.fail(format!(
                            "{}",
                            error
                        ));
                        progress.end();
                        MenuResult::Failed(error)
                    }
                }
            }
            Ok(false) => MenuResult::Aborted,
            Err(error) => match error {
                InquireError::OperationCanceled | InquireError::OperationInterrupted => MenuResult::Aborted,
                _ => MenuResult::Failed(error.into())
            },
                }
    }
}
