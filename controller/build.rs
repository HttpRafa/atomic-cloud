const PROTO_PATH: &str = "./proto/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile(
            &[format!("{}/control.proto", PROTO_PATH)],
            &[PROTO_PATH],
        )?;
    Ok(())
}