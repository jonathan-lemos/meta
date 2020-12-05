use serde::Deserialize;
use std::slice::Iter;

#[derive(Deserialize)]
struct CargoToml {
    package: Package;
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>
}

static TOML: &'static str = include_str!("../../Cargo.toml");
static PARSED: CargoToml = toml::from_str(TOML)
    .expect("The Cargo.toml used to create the program is malformed. This is theoretically impossible as Rust should stop compilation if Cargo.toml is invalid.");

pub fn version() -> &'static str {
    &PARSED.package.version
}

pub fn program_name() -> &'static str {
    &PARSED.package.name
}

pub fn authors() -> Iter<'static, String> {
    PARSED.package.authors.iter()
}

pub fn description() -> &'static str {
    &PARSED.package.description
}