use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::TempDir;

use utils::CommandExt;

mod utils;

fn check(schema: &str) {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("schema.vs");

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
}

include!("utils/test_schemas.inc.rs");
