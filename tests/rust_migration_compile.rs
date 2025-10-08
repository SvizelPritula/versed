use std::{env, fs, path::Path, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

mod utils;

const MOD_CONTENT: &str = indoc! {"
    #![allow(unused_imports)]

    use v1 as _;
    use v2 as _;
    use migrations::v2::upgrade;

    fn main() {}
"};

fn compile_schema(dir: &Path, name: &str, content: &str) {
    let path = dir.join(format!("{name}.vs"));
    fs::write(&path, content).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg(path)
        .arg(dir)
        .run_and_check();
}

fn check(old: &str, new: &str) {
    let mut dir = tempdir().unwrap();
    dir.disable_cleanup(true);

    let mod_path = dir.path().join("mod.rs");
    fs::write(&mod_path, MOD_CONTENT).unwrap();

    compile_schema(dir.path(), "old", old);
    compile_schema(dir.path(), "new", new);

    let migration = format!("{old}\n{new}");
    let migration_path = dir.path().join("schema.vsm");
    fs::write(&migration_path, migration).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("migration")
        .arg(migration_path)
        .arg(dir.path())
        .run_and_check();

    Command::new("rustc")
        .arg(mod_path)
        .arg("--out-dir")
        .arg(dir.path())
        .run_and_check();
}

#[test]
fn no_change() {
    let schema = indoc! {r#"
        Post = #1 struct {
            title: #2 string,
            content: #3 string,
            keywords: #4 [#5 string],
            visibility: #6 Visibility,
        };

        Visibility = #7 enum {
            public #8,
            restricted: #9 struct {
                users: #10 [#11 int]
            },
            private #12,
        };
    "#};

    check(
        &format!("version v1;\n\n{schema}"),
        &format!("version v2;\n\n{schema}"),
    );
}

#[test]
fn struct_change_field() {
    check(
        indoc! {r#"
            version v1;

            User = #1 struct {
                name: #2 string,
                age: #3 int,
            };
        "#},
        indoc! {r#"
            version v2;

            User = #1 struct {
                name: #2 string,
                age: #3 enum { some: int, none },
            };
        "#},
    );
}

#[test]
fn struct_add_field() {
    check(
        indoc! {r#"
            version v1;

            User = #1 struct {
                name: #2 string,
                age: #3 int,
            };
        "#},
        indoc! {r#"
            version v2;

            User = #1 struct {
                name: #2 string,
                age: #3 int,
                password: string,
            };
        "#},
    );
}

#[test]
fn change_identifier_target() {
    check(
        indoc! {r#"
            version v1;

            A = #1 struct {
                field: #2 B,
            };

            B = #3 int;
        "#},
        indoc! {r#"
            version v2;

            A = #1 struct {
                field: #2 B,
            };

            B = #3 string;
        "#},
    );
}
