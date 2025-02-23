use anyhow::Result;
use loading::Loading;
use start::ScreenMenu;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::manage::server, EstablishedConnection},
    profile::{Profile, Profiles},
};

mod start;

pub struct OpenScreenMenu;

struct Data {
    servers: Vec<server::Short>,
}

impl OpenScreenMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all required data from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match MenuUtils::select_no_help(
                    "Select the server hows screen you want to see?",
                    data.servers.clone(),
                ) {
                    Ok(server) => {
                        let progress = Loading::default();
                        progress.text("Subscribing to screen...");

                        match connection.client.subscribe_to_screen(&server.id).await {
                            Ok(stream) => {
                                progress.success("Subscribed successfully 👍.");
                                progress.end();
                                ScreenMenu::show(
                                    profile,
                                    connection,
                                    profiles,
                                    &server.name,
                                    &server.id,
                                    stream,
                                )
                                .await
                            }
                            Err(error) => {
                                progress.fail(format!("{error}"));
                                progress.end();
                                MenuResult::Failed(error)
                            }
                        }
                    }
                    Err(error) => MenuUtils::handle_error(error),
                }
            }
            Err(error) => {
                progress.fail(format!("{error}"));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let servers = connection.client.get_servers().await?;
        Ok(Data { servers })
    }
}
