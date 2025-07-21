pub mod connection;
pub mod known_host;
mod verifier;

pub mod proto {
    pub mod manage {
        #![allow(dead_code, clippy::all, clippy::pedantic)]
        use tonic::include_proto;

        include_proto!("manage");
    }

    pub mod common {
        #![allow(dead_code, clippy::all, clippy::pedantic)]
        use tonic::include_proto;

        include_proto!("common");
    }
}
