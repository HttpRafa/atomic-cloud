use inquire::Select;
use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct GetCloudletMenu;

impl GetCloudletMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Getting all available cloudlets from controller \"{}\"",
            profile.name
        ));
        
        match connection.client.get_cloudlets().await {
            Ok(cloudlets) => {
                progress.success("Data received 👍");
                progress.end();
                match Select::new("From what cloudlet do want more information?", cloudlets)
                    .prompt()
                {
                    Ok(cloudlet) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Getting information from controller \"{}\" about cloudlet \"{}\"",
                            profile.name,
                            cloudlet
                        ));
                
                        match connection.client.get_cloudlet(&cloudlet).await {
                            Ok(cloudlet) => {
                                progress.success("Data received 👍");
                                progress.end();
                                info!("   <blue>🖥  <b>Cloudlet Info</>");
                                info!("      <green><b>Name</>: {}", cloudlet.name);
                                info!("      <green><b>Driver</>: {}", cloudlet.driver);
                                if let Some(memory) = cloudlet.memory {
                                    info!("      <green><b>Memory</>: {} MiB", memory);
                                }
                                if let Some(max_allocations) = cloudlet.max_allocations {
                                    info!("      <green><b>Max Allocations</>: {} Units", max_allocations);
                                }
                                if let Some(child) = cloudlet.child {
                                    info!("      <green><b>Child Node</>: {}", child);
                                }
                                info!("      <green><b>Controller Address</>: {}", cloudlet.controller_address);
                                MenuResult::Success
                            }
                            Err(err) => {
                                progress.fail(format!(
                                    "Ops. Something went wrong while getting the required version information from the controller | {}",
                                    err
                                ));
                                progress.end();
                                MenuResult::Failed
                            }
                        }
                    }
                    Err(_) => MenuResult::Aborted
                }
            }
            Err(err) => {
                progress.fail(format!(
                    "Ops. Something went wrong while getting the required version information from the controller | {}",
                    err
                ));
                progress.end();
                MenuResult::Failed
            }
        }
    }
}