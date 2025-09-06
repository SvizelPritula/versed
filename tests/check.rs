use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::NamedTempFile;

use utils::CommandExt;

mod utils;

fn check(schema: &str) {
    let file = NamedTempFile::new().unwrap();

    fs::write(file.path(), schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("check")
        .arg(file.path())
        .run_and_check();
}

include!("utils/test_schemas.inc.rs");

#[test]
fn unnormalized() {
    check(indoc! {"
        version v1;

        n\u{303} = enum { value: \u{f1}, none };
    "});
}
