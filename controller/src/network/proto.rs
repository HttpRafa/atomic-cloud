pub mod common {
    #![allow(dead_code, clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    // TODO: Remove when issue https://github.com/hyperium/tonic/issues/2388 is fixed
    use std::primitive::u32;

    include_proto!("common");
}

pub mod manage {
    #![allow(dead_code, clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    // TODO: Remove when issue https://github.com/hyperium/tonic/issues/2388 is fixed
    use std::primitive::u32;

    include_proto!("manage");
}

pub mod client {
    #![allow(dead_code, clippy::all, clippy::pedantic)]
    use tonic::include_proto;

    // TODO: Remove when issue https://github.com/hyperium/tonic/issues/2388 is fixed
    use std::primitive::{bool, u32};

    include_proto!("client");
}
