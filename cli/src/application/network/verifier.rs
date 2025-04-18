use std::sync::Arc;

use futures::executor::block_on;
use rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{ServerName, UnixTime},
    DigitallySignedStruct, Error, SignatureScheme,
};
use sha2::{Digest, Sha256};
use tonic::transport::CertificateDer;

use super::known_host::manager::KnownHosts;

#[derive(Debug)]
pub struct FirstUseVerifier(CryptoProvider, Arc<KnownHosts>);

impl FirstUseVerifier {
    pub fn new(provider: CryptoProvider, known_hosts: Arc<KnownHosts>) -> Self {
        Self(provider, known_hosts)
    }
}

impl ServerCertVerifier for FirstUseVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        let mut hasher = Sha256::new();
        hasher.update(end_entity);
        let fingerprint = hasher.finalize().to_vec();

        if block_on(self.1.is_trusted(&server_name.to_str(), &fingerprint))
            .map_err(|error| Error::General(format!("Failed to check known hosts: {error}")))?
        {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(Error::General("Server certificate not trusted".to_string()))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
