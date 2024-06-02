const PROTO_PATH: &str = "../structure/proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile(
            &[format!("{}/admin.proto", PROTO_PATH), format!("{}/server.proto", PROTO_PATH)],
            &[PROTO_PATH],
        )?;
    Ok(())
}