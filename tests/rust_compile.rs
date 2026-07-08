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

const EXTRA_ARGS: [&[&str]; 2] = [&[], &["--derive", "Eq", "--derive", "PartialEq"]];

fn check(schema: &str) {
    for extra_args in EXTRA_ARGS {
        let dir = tempdir().unwrap();

        let mod_path = dir.path().join("mod.rs");
        fs::write(&mod_path, MOD_CONTENT).unwrap();

        let schema_path = dir.path().join("schema.vd");
        fs::write(&schema_path, schema).unwrap();

        Command::new(env!("CARGO_BIN_EXE_versed"))
            .arg("rust")
            .arg("types")
            .args(extra_args)
            .arg(schema_path)
            .arg(dir.path())
            .run_and_check();

        Command::new("rustc")
            .arg(mod_path)
            .arg("--out-dir")
            .arg(dir.path())
            .run_and_check();
    }
}

include!("utils/test_schemas.inc.rs");
include!("utils/rust_schemas.inc.rs");
