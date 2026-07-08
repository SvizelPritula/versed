use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

mod utils;

const MOD_CONTENT: &str = concat!(
    "#[allow(unused_imports)]\n",
    "use v1 as _;\n",
    "fn main() {}\n"
);

const MANIFEST_CONTENT: &str = indoc! {r#"
    [package]
    name = "versed_fixture"
    version = "0.1.0"
    edition = "2024"

    [[bin]]
    name = "versed_fixture"
    path = "src/mod.rs"

    [dependencies]
    serde = { version = "1.0.219", features = ["derive"] }
"#};

fn check(schema: &str) {
    let dir = tempdir().unwrap();

    let manifest_path = dir.path().join("Cargo.toml");
    fs::write(&manifest_path, MANIFEST_CONTENT).unwrap();

    let src_path = dir.path().join("src");
    fs::create_dir(&src_path).unwrap();

    let mod_path = src_path.join("mod.rs");
    fs::write(&mod_path, MOD_CONTENT).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg("--serde")
        .arg(schema_path)
        .arg(src_path)
        .run_and_check();

    Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .run_and_check();
}

include!("utils/test_schemas.inc.rs");
include!("utils/rust_schemas.inc.rs");
