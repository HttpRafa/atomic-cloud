pub mod common {
    use tonic::include_proto;

    include_proto!("common");
}

pub mod manage {
    use tonic::include_proto;

    include_proto!("manage");
}

pub mod client {
    use tonic::include_proto;

    include_proto!("client");
}
