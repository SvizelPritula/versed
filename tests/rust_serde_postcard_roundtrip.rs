use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

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
    postcard = { version = "1.1.3", features = ["use-std"] }
"#};

fn check(schema: &str, r#type: &str, value: &str) {
    let dir = tempdir().unwrap();

    let manifest_path = dir.path().join("Cargo.toml");
    fs::write(&manifest_path, MANIFEST_CONTENT).unwrap();

    let src_path = dir.path().join("src");
    fs::create_dir(&src_path).unwrap();

    let mod_path = src_path.join("mod.rs");
    let content = format!(
        indoc! {r#"
            #![recursion_limit = "256"]
            #[allow(unused_imports)]
            fn main() {{
                let before: {type} = {value};
                let payload = postcard::to_stdvec(&before).unwrap();
                let after: {type} = postcard::from_bytes(&payload).unwrap();
                assert_eq!(before, after);
            }}
        "#},
        value = value,
        r#type = r#type,
    );
    fs::write(&mod_path, content).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg("--serde")
        .arg("--serde-external-tag")
        .arg("--derive")
        .arg("PartialEq")
        .arg(schema_path)
        .arg(src_path)
        .run_and_check();

    Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .run_and_check();
}

include!("utils/roundtrip_schemas.inc.rs");
