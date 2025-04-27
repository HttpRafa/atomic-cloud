use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use group::{CreateGroupTask, GetGroupTask, GetGroupsTask, UpdateGroupTask};
use node::{CreateNodeTask, GetNodeTask, GetNodesTask, UpdateNodeTask};
use plugin::GetPluginsTask;
use power::RequestStopTask;
use resource::{DeleteResourceTask, SetResourceTask};
use server::{GetServerTask, GetServersTask, ScheduleServerTask};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use transfer::TransferUsersTask;
use user::GetUsersTask;
use uuid::Uuid;

use crate::{
    application::{
        auth::AuthType,
        group::{ScalingPolicy, StartConstraints},
        node::Capabilities,
        server::{DiskRetention, FallbackPolicy, Resources, Spec},
        user::transfer::TransferTarget,
        Shared, TaskSender,
    },
    task::Task,
    VERSION,
};

use super::proto::{
    common::notify::PowerEvent,
    manage::{
        self,
        manage_service_server::ManageService,
        resource::{Category, DelReq, SetReq},
        screen::{Lines, WriteReq},
        transfer::{target::Type, TransferReq},
    },
};

mod group;
mod node;
mod plugin;
mod power;
mod resource;
mod server;
pub mod transfer;
mod user;

pub type ScreenLines = Lines;

pub struct ManageServiceImpl(pub TaskSender, pub Arc<Shared>);

#[async_trait]
impl ManageService for ManageServiceImpl {
    type SubscribeToScreenStream = ReceiverStream<Result<Lines, Status>>;
    type SubscribeToPowerEventsStream = ReceiverStream<Result<PowerEvent, Status>>;

    // Power
    async fn request_stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(RequestStopTask()))
            })
            .await?,
        ))
    }

    // Resource
    async fn set_resource(&self, request: Request<SetReq>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let Ok(category) = Category::try_from(request.category) else {
                    return Err(Status::invalid_argument("Invalid category provided"));
                };

                Ok(Box::new(SetResourceTask(
                    category,
                    request.id,
                    request.active,
                )))
            })
            .await?,
        ))
    }
    async fn delete_resource(&self, request: Request<DelReq>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let Ok(category) = Category::try_from(request.category) else {
                    return Err(Status::invalid_argument("Invalid category provided"));
                };

                Ok(Box::new(DeleteResourceTask(category, request.id)))
            })
            .await?,
        ))
    }

    // Plugin
    async fn get_plugins(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::plugin::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::plugin::List, _, _>(
                AuthType::User,
                &self.0,
                request,
                |_, _| Ok(Box::new(GetPluginsTask())),
            )
            .await?,
        ))
    }

    // Node
    async fn create_node(
        &self,
        request: Request<manage::node::Detail>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let capabilities = match request.capabilities {
                    Some(capabilities) => {
                        Capabilities::new(capabilities.memory, capabilities.max, capabilities.child)
                    }
                    None => return Err(Status::invalid_argument("No capabilities provided")),
                };
                let controller = request
                    .ctrl_addr
                    .parse()
                    .map_err(|_| Status::invalid_argument("Invalid controller address provided"))?;
                let plugin = request.plugin;

                Ok(Box::new(CreateNodeTask(
                    request.name,
                    plugin,
                    capabilities,
                    controller,
                )))
            })
            .await?,
        ))
    }
    async fn update_node(
        &self,
        request: Request<manage::node::UpdateReq>,
    ) -> Result<Response<manage::node::Detail>, Status> {
        Ok(Response::new(
            Task::execute::<manage::node::Detail, _, _>(
                AuthType::User,
                &self.0,
                request,
                |request, _| {
                    let request = request.into_inner();

                    let capabilities = request.capabilities.map(|capabilities| {
                        Capabilities::new(capabilities.memory, capabilities.max, capabilities.child)
                    });
                    let controller = {
                        match request.ctrl_addr {
                            Some(addr) => {
                                let url = addr.parse();

                                match url {
                                    Ok(url) => Some(url),
                                    Err(_) => {
                                        return Err(Status::invalid_argument(
                                            "Invalid controller address provided",
                                        ))
                                    }
                                }
                            }
                            None => None,
                        }
                    };

                    Ok(Box::new(UpdateNodeTask(
                        request.name,
                        capabilities,
                        controller,
                    )))
                },
            )
            .await?,
        ))
    }
    async fn get_node(
        &self,
        request: Request<String>,
    ) -> Result<Response<manage::node::Detail>, Status> {
        Ok(Response::new(
            Task::execute::<manage::node::Detail, _, _>(
                AuthType::User,
                &self.0,
                request,
                |request, _| {
                    let request = request.into_inner();

                    Ok(Box::new(GetNodeTask(request)))
                },
            )
            .await?,
        ))
    }
    async fn get_nodes(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::node::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::node::List, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetNodesTask()))
            })
            .await?,
        ))
    }

    // Group
    async fn create_group(
        &self,
        request: Request<manage::group::Detail>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let constraints = match request.constraints {
                    Some(constrains) => {
                        StartConstraints::new(constrains.min, constrains.max, constrains.prio)
                    }
                    None => return Err(Status::invalid_argument("No constraints provided")),
                };

                let scaling = match request.scaling {
                    Some(scaling) => ScalingPolicy::new(
                        scaling.enabled,
                        scaling.start_threshold,
                        scaling.stop_empty,
                    ),
                    None => return Err(Status::invalid_argument("No scaling policy provided")),
                };

                let resources = match request.resources {
                    Some(resources) => Resources::new(
                        resources.memory,
                        resources.swap,
                        resources.cpu,
                        resources.io,
                        resources.disk,
                        resources.ports,
                    ),
                    None => return Err(Status::invalid_argument("No resources provided")),
                };

                let spec = match request.spec {
                    Some(spec) => {
                        let image = spec.img;
                        let max_players = spec.max_players;
                        let settings = spec
                            .settings
                            .iter()
                            .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                            .collect();
                        let environment = spec
                            .env
                            .iter()
                            .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                            .collect();
                        let disk_retention = if let Some(retention) = spec.retention {
                            match manage::server::DiskRetention::try_from(retention) {
                                Ok(manage::server::DiskRetention::Permanent) => {
                                    DiskRetention::Permanent
                                }
                                Ok(manage::server::DiskRetention::Temporary) => {
                                    DiskRetention::Temporary
                                }
                                Err(_) => {
                                    return Err(Status::invalid_argument(
                                        "Invalid disk retention provided",
                                    ))
                                }
                            }
                        } else {
                            DiskRetention::Temporary
                        };
                        let fallback = if let Some(fallback) = spec.fallback {
                            FallbackPolicy::new(true, fallback.prio)
                        } else {
                            FallbackPolicy::default()
                        };
                        Spec::new(
                            settings,
                            environment,
                            disk_retention,
                            image,
                            max_players,
                            fallback,
                        )
                    }
                    None => return Err(Status::invalid_argument("No spec provided")),
                };

                let nodes = request.nodes;

                Ok(Box::new(CreateGroupTask(
                    request.name,
                    constraints,
                    scaling,
                    resources,
                    spec,
                    nodes,
                )))
            })
            .await?,
        ))
    }
    async fn update_group(
        &self,
        request: Request<manage::group::UpdateReq>,
    ) -> Result<Response<manage::group::Detail>, Status> {
        Ok(Response::new(
            Task::execute::<manage::group::Detail, _, _>(
                AuthType::User,
                &self.0,
                request,
                |request, _| {
                    let request = request.into_inner();

                    let nodes = request.nodes.map(|list| list.nodes);

                    let constraints = request.constraints.map(|constraints| {
                        StartConstraints::new(constraints.min, constraints.max, constraints.prio)
                    });

                    let scaling = request.scaling.map(|scaling| {
                        ScalingPolicy::new(true, scaling.start_threshold, scaling.stop_empty)
                    });

                    let resources = request.resources.map(|resources| {
                        Resources::new(
                            resources.memory,
                            resources.swap,
                            resources.cpu,
                            resources.io,
                            resources.disk,
                            resources.ports,
                        )
                    });

                    let spec = match request.spec {
                        Some(spec) => {
                            let image = spec.img;
                            let max_players = spec.max_players;
                            let settings = spec
                                .settings
                                .iter()
                                .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                                .collect();
                            let environment = spec
                                .env
                                .iter()
                                .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                                .collect();
                            let disk_retention = if let Some(retention) = spec.retention {
                                match manage::server::DiskRetention::try_from(retention) {
                                    Ok(manage::server::DiskRetention::Permanent) => {
                                        DiskRetention::Permanent
                                    }
                                    Ok(manage::server::DiskRetention::Temporary) => {
                                        DiskRetention::Temporary
                                    }
                                    Err(_) => {
                                        return Err(Status::invalid_argument(
                                            "Invalid disk retention provided",
                                        ))
                                    }
                                }
                            } else {
                                DiskRetention::Temporary
                            };
                            let fallback = if let Some(fallback) = spec.fallback {
                                FallbackPolicy::new(true, fallback.prio)
                            } else {
                                FallbackPolicy::default()
                            };
                            Some(Spec::new(
                                settings,
                                environment,
                                disk_retention,
                                image,
                                max_players,
                                fallback,
                            ))
                        }
                        None => None,
                    };

                    Ok(Box::new(UpdateGroupTask(
                        request.name,
                        constraints,
                        scaling,
                        resources,
                        spec,
                        nodes,
                    )))
                },
            )
            .await?,
        ))
    }
    async fn get_group(
        &self,
        request: Request<String>,
    ) -> Result<Response<manage::group::Detail>, Status> {
        Ok(Response::new(
            Task::execute::<manage::group::Detail, _, _>(
                AuthType::User,
                &self.0,
                request,
                |request, _| {
                    let request = request.into_inner();

                    Ok(Box::new(GetGroupTask(request)))
                },
            )
            .await?,
        ))
    }
    async fn get_groups(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::group::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::group::List, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetGroupsTask()))
            })
            .await?,
        ))
    }

    // Server
    async fn schedule_server(
        &self,
        request: Request<manage::server::Proposal>,
    ) -> Result<Response<String>, Status> {
        Ok(Response::new(
            Task::execute::<String, _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let resources = match request.resources {
                    Some(resources) => Resources::new(
                        resources.memory,
                        resources.swap,
                        resources.cpu,
                        resources.io,
                        resources.disk,
                        resources.ports,
                    ),
                    None => return Err(Status::invalid_argument("No resources provided")),
                };

                let spec = match request.spec {
                    Some(spec) => {
                        let image = spec.img;
                        let max_players = spec.max_players;
                        let settings = spec
                            .settings
                            .iter()
                            .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                            .collect();
                        let environment = spec
                            .env
                            .iter()
                            .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                            .collect();
                        let disk_retention = if let Some(retention) = spec.retention {
                            match manage::server::DiskRetention::try_from(retention) {
                                Ok(manage::server::DiskRetention::Permanent) => {
                                    DiskRetention::Permanent
                                }
                                Ok(manage::server::DiskRetention::Temporary) => {
                                    DiskRetention::Temporary
                                }
                                Err(_) => {
                                    return Err(Status::invalid_argument(
                                        "Invalid disk retention provided",
                                    ))
                                }
                            }
                        } else {
                            DiskRetention::Temporary
                        };
                        let fallback = if let Some(fallback) = spec.fallback {
                            FallbackPolicy::new(true, fallback.prio)
                        } else {
                            FallbackPolicy::default()
                        };
                        Spec::new(
                            settings,
                            environment,
                            disk_retention,
                            image,
                            max_players,
                            fallback,
                        )
                    }
                    None => return Err(Status::invalid_argument("No spec provided")),
                };

                Ok(Box::new(ScheduleServerTask(
                    request.prio,
                    request.name.clone(),
                    request.node,
                    resources,
                    spec,
                )))
            })
            .await?,
        ))
    }
    async fn get_server(
        &self,
        request: Request<String>,
    ) -> Result<Response<manage::server::Detail>, Status> {
        Ok(Response::new(
            Task::execute::<manage::server::Detail, _, _>(
                AuthType::User,
                &self.0,
                request,
                |request, _| {
                    let request = request.into_inner();

                    let Ok(uuid) = Uuid::parse_str(&request) else {
                        return Err(Status::invalid_argument("Invalid UUID provided"));
                    };

                    Ok(Box::new(GetServerTask(uuid)))
                },
            )
            .await?,
        ))
    }
    async fn get_servers(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::server::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::server::List, _, _>(
                AuthType::User,
                &self.0,
                request,
                |_, _| Ok(Box::new(GetServersTask())),
            )
            .await?,
        ))
    }

    // Screen
    async fn write_to_screen(&self, request: Request<WriteReq>) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let Ok(uuid) = Uuid::from_str(&request.id) else {
            return Err(Status::invalid_argument("Invalid UUID provided"));
        };

        match self.1.screens.write(&uuid, &request.data).await?.await {
            Ok(Err(error)) => Err(error.into()),
            Err(error) => Err(Status::internal(error.to_string())),
            Ok(_) => Ok(Response::new(())),
        }
    }
    async fn subscribe_to_screen(
        &self,
        request: Request<String>,
    ) -> Result<Response<Self::SubscribeToScreenStream>, Status> {
        let Ok(uuid) = Uuid::from_str(&request.into_inner()) else {
            return Err(Status::invalid_argument("Invalid UUID provided"));
        };

        Ok(Response::new(self.1.screens.subscribe_screen(&uuid).await?))
    }

    // User
    async fn get_users(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::user::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::user::List, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetUsersTask()))
            })
            .await?,
        ))
    }

    // Transfer
    async fn transfer_users(&self, request: Request<TransferReq>) -> Result<Response<u32>, Status> {
        Ok(Response::new(
            Task::execute::<u32, _, _>(AuthType::User, &self.0, request, |request, auth| {
                let request = request.into_inner();

                let target = match request.target {
                    Some(target) => match Type::try_from(target.r#type) {
                        Ok(r#type) => match (target.target, r#type) {
                            (Some(target), Type::Group) => TransferTarget::Group(target),
                            (Some(target), Type::Server) => {
                                TransferTarget::Server(match Uuid::from_str(&target) {
                                    Ok(uuid) => uuid,
                                    Err(_) => {
                                        return Err(Status::invalid_argument(
                                            "Invalid UUID provided",
                                        ))
                                    }
                                })
                            }
                            (None, Type::Fallback) => TransferTarget::Fallback,
                            _ => {
                                return Err(Status::invalid_argument(
                                    "Invalid target type combination",
                                ))
                            }
                        },
                        Err(_) => {
                            return Err(Status::invalid_argument("Invalid target type provided"))
                        }
                    },
                    None => return Err(Status::invalid_argument("Missing target")),
                };
                let uuids = request
                    .ids
                    .into_iter()
                    .map(|id| match Uuid::from_str(&id) {
                        Ok(uuid) => Ok(uuid),
                        Err(_) => Err(Status::invalid_argument("Invalid UUID provided")),
                    })
                    .collect::<Result<Vec<Uuid>, _>>()?;

                Ok(Box::new(TransferUsersTask(auth, uuids, target)))
            })
            .await?,
        ))
    }

    // Version info
    async fn get_proto_ver(&self, _request: Request<()>) -> Result<Response<u32>, Status> {
        Ok(Response::new(VERSION.protocol))
    }
    async fn get_ctrl_ver(&self, _request: Request<()>) -> Result<Response<String>, Status> {
        Ok(Response::new(format!("{VERSION}")))
    }

    // Notify operations
    async fn subscribe_to_power_events(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::SubscribeToPowerEventsStream>, Status> {
        Err(Status::unimplemented("Not implemented yet"))
    }
}
