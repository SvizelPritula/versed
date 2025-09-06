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

fn check(schema: &str) {
    let dir = tempdir().unwrap();

    let mod_path = dir.path().join("mod.rs");
    fs::write(&mod_path, MOD_CONTENT).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg(schema_path)
        .arg(dir.path())
        .run_and_check();

    Command::new("rustc")
        .arg(mod_path)
        .arg("--out-dir")
        .arg(dir.path())
        .run_and_check();
}

include!("utils/test_schemas.inc.rs");

#[test]
fn rust_type_idents() {
    check(indoc! {r#"
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
    check(indoc! {r#"
        version v1;

        "struct" = struct {
            box: int,
            self: int,
        };
    "#});
}
