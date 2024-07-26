const PROTO_PATH: &str = "../protocol/grpc";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_client(false).compile(
        &[
            format!("{}/admin/admin.proto", PROTO_PATH),
            format!("{}/server/server.proto", PROTO_PATH),
        ],
        &[PROTO_PATH],
    )?;
    Ok(())
}
