use std::{env, fs, process::Command};

use tempfile::tempdir;

const MOD_CONTENT: &str = "let a: object = v1;\n";

fn translate_and_check(schema: &str) {
    let dir = tempdir().unwrap();

    let index_path = dir.path().join("index.ts");
    fs::write(&index_path, MOD_CONTENT).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("typescript")
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

    let output = Command::new("tsc")
        .arg("--module")
        .arg("esnext")
        .arg(index_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Error running tsc:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

include!("utils/test_schemas.inc.rs");

#[test]
fn typescript_type_idents() {
    translate_and_check(indoc! {"
        version v1;

        Map = unit;
        String = struct { a: string };
        Partial = int;
        Lowercase = string;
    "});
}

#[test]
fn keyword_idents() {
    translate_and_check(indoc! {"
        version v1;

        class = struct {
            let: int,
            any: int,
        };

        let = int;
        type = int;
        any = int;
        of = int;
    "});
}
