pub mod common {
    #![allow(dead_code, clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    include_proto!("common");
}

pub mod manage {
    #![allow(dead_code, clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    include_proto!("manage");
}

pub mod client {
    #![allow(dead_code, clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    include_proto!("client");
}
