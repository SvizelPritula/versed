use std::{env, fs, process::Command};

use tempfile::tempdir;

const MOD_CONTENT: &str = concat!(
    "#[allow(unused_imports)]\n",
    "use v1 as _;\n",
    "fn main() {}\n"
);

fn translate_and_check(schema: &str) {
    let dir = tempdir().unwrap();

    let mod_path = dir.path().join("mod.rs");
    fs::write(&mod_path, MOD_CONTENT).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg(schema_path)
        .arg(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Error running versed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new("rustc")
        .arg(mod_path)
        .arg("--out-dir")
        .arg(dir.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Error running rustc:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

include!("utils/test_schemas.inc.rs");

#[test]
fn rust_type_idents() {
    translate_and_check(indoc! {r#"
        version v1;

        String = unit;

        "" = struct {
            vec: struct {}
        };

        Struct = struct {
            a: string,
            b: [int],
        };
    "#});
}

#[test]
fn keyword_idents() {
    translate_and_check(indoc! {r#"
        version v1;

        "struct" = struct {
            box: int,
            self: int,
        };
    "#});
}
