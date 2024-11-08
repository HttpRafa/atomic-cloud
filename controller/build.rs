use std::{
    env,
    fs::{self, File},
    io::Write,
};

const PROTO_PATH: &str = "../protocol/grpc";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_build_info("Alpha");
    generate_grpc_code()?;
    Ok(())
}

fn generate_build_info(stage: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file = File::create(format!("{}/build_info.rs", out_dir)).unwrap();

    let commit = env::var("CURRENT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let build = env::var("CURRENT_BUILD").unwrap_or_else(|_| "0".to_string());

    let version = get_version_info().expect("Unable to get version information");

    writeln!(file, "use common::version::{{Stage, Version}};").unwrap();
    writeln!(file, "pub const VERSION: Version = Version {{").unwrap();
    writeln!(file, "    major: {},", version.0).unwrap();
    writeln!(file, "    minor: {},", version.1).unwrap();
    writeln!(file, "    patch: {},", version.2).unwrap();
    writeln!(file, "    build: {},", build).unwrap();
    writeln!(file, "    commit: \"{}\",", commit).unwrap();
    writeln!(file, "    stage: Stage::{},", stage).unwrap();
    writeln!(file, "}};").unwrap();
}

fn get_version_info() -> Result<(u16, u16, u16), Box<dyn std::error::Error>> {
    let cargo_toml_content = fs::read_to_string("Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let version_str = cargo_toml["package"]["version"]
        .as_str()
        .ok_or("Unable to get version from Cargo.toml")?;

    let version_parts: Vec<u16> = version_str
        .split('.')
        .map(|v| v.parse().map_err(|_| "Invalid version part"))
        .collect::<Result<Vec<_>, _>>()?;

    if version_parts.len() == 3 {
        Ok((version_parts[0], version_parts[1], version_parts[2]))
    } else {
        Err("Version must have three parts".into())
    }
}

fn generate_grpc_code() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile_protos(
            &[
                format!("{}/admin/admin.proto", PROTO_PATH),
                format!("{}/unit/unit.proto", PROTO_PATH),
            ],
            &[PROTO_PATH],
        )?;
    Ok(())
}
