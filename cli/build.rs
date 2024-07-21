const PROTO_PATH: &str = "../protocol/grpc";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(&[format!("{}/admin.proto", PROTO_PATH)], &[PROTO_PATH])?;
    Ok(())
}
