use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::TempDir;

use utils::CommandExt;

mod utils;

fn check_with_options(schema: &str, check_roundtrip: bool) {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("schema.vs");
    let old_file = dir.path().join("schema.vs.old");
    let migration_file = dir.path().join("schema.vsm");

    fs::write(&file, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("migration")
        .arg("begin")
        .arg(&file)
        .run_and_check();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("check")
        .arg(&file)
        .run_and_check();

    assert!(fs::exists(&old_file).unwrap());

    assert_eq!(
        fs::read_to_string(&old_file).unwrap(),
        fs::read_to_string(&file).unwrap(),
    );

    let content = fs::read_to_string(&file).unwrap();
    let content = content.replace("version v1;", "version v2;");
    fs::write(&file, content).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("migration")
        .arg("finish")
        .arg(&file)
        .arg(&migration_file)
        .run_and_check();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("check")
        .arg(&file)
        .run_and_check();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("migration")
        .arg("check")
        .arg(&migration_file)
        .run_and_check();

    assert!(!fs::exists(&old_file).unwrap());

    if check_roundtrip {
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            schema.replace("version v1;", "version v2;")
        );
    }
}

fn check(schema: &str) {
    check_with_options(schema, !schema.contains('#'));
}

include!("utils/test_schemas.inc.rs");

#[test]
fn pre_annotated() {
    check_with_options(
        indoc! {"
            version v1;

            Result = #1 enum {
                error: #2 string,
                ok #3
            };
        "},
        false,
    );
}

#[test]
fn partially_annotated() {
    check_with_options(
        indoc! {"
            version v1;

            Result = enum {
                error: #1 string,
                ok
            };
        "},
        false,
    );
}
