#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("client");
}