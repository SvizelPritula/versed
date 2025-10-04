use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

use crate::utils::TSC_OPTIONS;

mod utils;

fn check_with_version(schema: &str, version: &str) {
    let dir = tempdir().unwrap();

    let index_path = dir.path().join("index.ts");
    let index_content = format!("let a: object = {version};\n");
    fs::write(&index_path, index_content).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("typescript")
        .arg("types")
        .arg(schema_path)
        .arg(dir.path())
        .run_and_check();

    Command::new("tsc")
        .args(TSC_OPTIONS)
        .arg(index_path)
        .run_and_check();
}

fn check(schema: &str) {
    check_with_version(schema, "v1");
}

include!("utils/test_schemas.inc.rs");

#[test]
fn typescript_type_idents() {
    check(indoc! {"
        version v1;

        Map = unit;
        String = struct { a: string };
        Partial = int;
        Lowercase = string;
    "});
}

#[test]
fn keyword_idents() {
    check(indoc! {"
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

#[test]
fn keyword_version() {
    check_with_version("version yield;", "yield_");
}
