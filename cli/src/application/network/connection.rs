use std::{str::FromStr, sync::Arc};

use color_eyre::eyre::Result;
use hyper::Uri;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};
use task::{spawn, ConnectTask};
use tokio::sync::RwLock;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tonic::{body::Body, Request};
use tower::ServiceBuilder;
use url::Url;

use crate::VERSION;

use super::{
    known_host::manager::KnownHosts, proto::manage::manage_service_client::ManageServiceClient,
    verifier::FirstUseVerifier,
};

pub mod task;
pub mod wrapper;

pub struct EstablishedConnection {
    /* Host */
    name: String,

    /* Connection */
    connection: RwLock<ManageServiceClient<Client<HttpsConnector<HttpConnector>, Body>>>,
    incompatible: bool,
    protocol: u32,

    /* Data */
    token: String,
}

impl EstablishedConnection {
    pub fn establish_new(
        name: String,
        url: Url,
        token: String,
        known_hosts: Arc<KnownHosts>,
    ) -> ConnectTask {
        spawn(async move {
            match EstablishedConnection::establish(
                name.clone(),
                url.clone(),
                token.clone(),
                known_hosts.clone(),
            )
            .await
            {
                Err(_) => {
                    // Wait for all TLS prompt to be resolved
                    known_hosts.requests.wait_for_empty().await;

                    EstablishedConnection::establish(
                        name,
                        url.clone(),
                        token.clone(),
                        known_hosts.clone(),
                    )
                    .await
                }
                Ok(connection) => Ok(connection),
            }
        })
    }

    async fn establish(
        name: String,
        url: Url,
        token: String,
        known_hosts: Arc<KnownHosts>,
    ) -> Result<Self> {
        let mut tls = ClientConfig::builder()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();
        let verifier = FirstUseVerifier::new(tls.crypto_provider().clone(), known_hosts);
        tls.dangerous().set_certificate_verifier(Arc::new(verifier));

        let mut http = HttpConnector::new();
        http.enforce_http(false);

        let connector = ServiceBuilder::new()
            .layer_fn(move |service| {
                HttpsConnectorBuilder::new()
                    .with_tls_config(tls.clone())
                    .https_or_http()
                    .enable_http2()
                    .wrap_connector(service)
            })
            .service(http);
        let client = Client::builder(TokioExecutor::new()).build(connector);

        let mut connection = EstablishedConnection {
            name,
            connection: RwLock::new(ManageServiceClient::with_origin(
                client,
                Uri::from_str(url.as_str())?,
            )),
            token: token.clone(),
            incompatible: false,
            protocol: 1,
        };
        let protocol = connection.get_proto_ver().await?;
        connection.protocol = protocol;
        connection.incompatible = protocol != VERSION.protocol;

        Ok(connection)
    }

    pub fn is_incompatible(&self) -> bool {
        self.incompatible
    }

    pub fn get_protocol(&self) -> u32 {
        self.protocol
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn create_request<T>(&self, data: T) -> Request<T> {
        let mut request = Request::new(data);

        // Add the token to the metadata
        request.metadata_mut().insert(
            "authorization",
            self.token
                .parse()
                .expect("Failed to convert token to header value"),
        );

        request
    }
}
