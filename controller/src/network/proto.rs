pub mod common {
    #![allow(clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    include_proto!("common");
}

pub mod manage {
    #![allow(clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    include_proto!("manage");
}

pub mod client {
    #![allow(clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    include_proto!("client");
}
