use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write as _,
};

use winresource::WindowsResource;

const PROTO_PATH: &str = "../protocol/grpc";

fn main() -> Result<(), Box<dyn Error>> {
    add_windows_resources();
    generate_build_info();
    generate_grpc_code()?;
    Ok(())
}

fn add_windows_resources() {
    if env::var("CARGO_CFG_TARGET_OS").expect("This variable should be set by cargo") == "windows" {
        let mut resource = WindowsResource::new();
        resource.set_icon("../images/icon.ico");
        resource
            .compile()
            .expect("Failed to compile windows resources");
    }
}

fn generate_build_info() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file = File::create(format!("{out_dir}/build_info.rs")).unwrap();

    let commit = env::var("CURRENT_COMMIT").unwrap_or_else(|_| "unknown".to_owned());
    let build = env::var("CURRENT_BUILD").unwrap_or_else(|_| "0".to_owned());

    let version = get_version_info().expect("Unable to get version information");
    let protocol_version =
        get_protocol_version_info().expect("Unable to get protocol version information");

    writeln!(file, "use common::version::{{Stage, Version}};").unwrap();
    writeln!(file, "pub const VERSION: Version = Version {{").unwrap();
    writeln!(file, "    major: {},", version.0).unwrap();
    writeln!(file, "    minor: {},", version.1).unwrap();
    writeln!(file, "    patch: {},", version.2).unwrap();
    writeln!(file, "    build: {build},").unwrap();
    writeln!(file, "    commit: \"{commit}\",").unwrap();
    writeln!(file, "    stage: Stage::{},", version.3).unwrap();
    writeln!(file, "    protocol: {protocol_version},").unwrap();
    writeln!(file, "}};").unwrap();
}

fn get_version_info() -> Result<(u16, u16, u16, String), Box<dyn core::error::Error>> {
    let cargo_toml_content = fs::read_to_string("Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let version_str = cargo_toml["package"]["version"]
        .as_str()
        .ok_or("Unable to get version from Cargo.toml")?;

    let version_parts: Vec<&str> = version_str.split('-').collect();
    let version_numbers: Vec<u16> = version_parts[0]
        .split('.')
        .map(|v| v.parse().map_err(|_| "Invalid version part"))
        .collect::<Result<Vec<_>, _>>()?;

    if version_numbers.len() == 3 {
        let stage = if version_parts.len() > 1 {
            version_parts[1][0..1].to_uppercase() + &version_parts[1][1..]
        } else {
            "Stable".to_owned()
        };
        Ok((
            version_numbers[0],
            version_numbers[1],
            version_numbers[2],
            stage,
        ))
    } else {
        Err("Version must have three parts".into())
    }
}

fn get_protocol_version_info() -> Result<u32, Box<dyn core::error::Error>> {
    let cargo_toml_content = fs::read_to_string("../Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let value = cargo_toml["workspace"]["metadata"]["protocol-version"]
        .as_integer()
        .map(|v| v as u32);
    value.ok_or("Unable to get protocol version from Cargo.toml".into())
}

fn generate_grpc_code() -> Result<(), Box<dyn core::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .type_attribute(".", "#[derive(serde::Serialize)]")
        .compile_protos(
            &[format!("{PROTO_PATH}/manage/service.proto")],
            &[PROTO_PATH],
        )?;
    Ok(())
}
