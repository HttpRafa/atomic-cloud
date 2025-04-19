use std::sync::Arc;

use sha2::{Digest, Sha256};
use tokio_rustls::rustls::{
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    pki_types::{ServerName, UnixTime},
    DigitallySignedStruct, Error, SignatureScheme,
};
use tonic::transport::CertificateDer;

use super::known_host::{manager::KnownHosts, requests::TrustResult};

#[derive(Debug)]
pub struct FirstUseVerifier(Arc<CryptoProvider>, Arc<KnownHosts>);

impl FirstUseVerifier {
    pub fn new(provider: Arc<CryptoProvider>, known_hosts: Arc<KnownHosts>) -> Self {
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

        match self.1.is_trusted(&server_name.to_str(), &fingerprint) {
            TrustResult::Trusted => Ok(ServerCertVerified::assertion()),
            TrustResult::HostDuplicate => Err(Error::General(
                "Cannot trust 2 certs for the same host".to_string(),
            )),
            TrustResult::Declined => {
                Err(Error::General("Server certificate not trusted".to_string()))
            }
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
