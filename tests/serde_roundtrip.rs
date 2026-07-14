use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

use crate::utils::{TSC_COMMAND, TSC_OPTIONS};

mod utils;

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
    serde_json = "1.0.143"
"#};

fn check(schema: &str, r#type: &str, value: &str) {
    let dir = tempdir().unwrap();

    let manifest_path = dir.path().join("Cargo.toml");
    fs::write(&manifest_path, MANIFEST_CONTENT).unwrap();

    let src_path = dir.path().join("src");
    fs::create_dir(&src_path).unwrap();

    let mod_content = format!(
        indoc! {r#"
            fn main() {{
                let user: {type} = {value};

                let content = serde_json::to_string_pretty(&user).unwrap();
                println!("{{}}", content);
            }}
        "#},
        value = value,
        r#type = r#type
    );

    let mod_path = src_path.join("mod.rs");
    fs::write(&mod_path, mod_content).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg("--serde")
        .arg(&schema_path)
        .arg(&src_path)
        .run_and_check();

    let json = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .run_and_check();

    let typescript_path = dir.path().join("typescript");
    fs::create_dir(&typescript_path).unwrap();

    let entrypoint_path = typescript_path.join("main.ts");
    let entrypoint_content = format!(
        "import {{ v1 }} from \"./index\";\nlet user: {type} = {json};\n",
        r#type = r#type.replace("::", ".")
    );
    fs::write(&entrypoint_path, entrypoint_content).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("typescript")
        .arg("types")
        .arg(&schema_path)
        .arg(&typescript_path)
        .run_and_check();

    Command::new(TSC_COMMAND)
        .args(TSC_OPTIONS)
        .arg(entrypoint_path)
        .run_and_check();
}

include!("utils/roundtrip_schemas.inc.rs");
