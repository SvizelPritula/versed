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
    let dir = tempdir().unwrap();

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

mod unchanged {
    use std::{fs, process::Command};

    use indoc::indoc;
    use tempfile::tempdir;

    use crate::utils::CommandExt;

    fn check(schema: &str) {
        let dir = tempdir().unwrap();
        let file = dir.path().join("schema.rs");

        fs::write(&file, schema).unwrap();

        Command::new(env!("CARGO_BIN_EXE_versed"))
            .arg("migration")
            .arg("begin")
            .arg(&file)
            .run_and_check();

        let schema = fs::read_to_string(&file).unwrap();

        super::check(&schema, &schema.replace("version v1;", "version v2;"));
    }

    include!("utils/test_schemas.inc.rs");

    #[test]
    fn complex_example() {
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

        super::check(
            &format!("version v1;\n\n{schema}"),
            &format!("version v2;\n\n{schema}"),
        );
    }
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
fn struct_remove_field() {
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
            };
        "#},
    );
}

#[test]
fn struct_rename_field() {
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
                full_name: #2 string,
                age: #3 int,
            };
        "#},
    );
}

#[test]
fn enum_change_variant() {
    check(
        indoc! {r#"
            version v1;

            Contact = #1 enum {
                email: #2 string,
                phone: #3 int,
            };
        "#},
        indoc! {r#"
            version v2;

            Contact = #1 enum {
                email: #2 string,
                phone: #3 string,
            };
        "#},
    );
}

#[test]
fn enum_add_variant() {
    check(
        indoc! {r#"
            version v1;

            Contact = #1 enum {
                email: #2 string,
                phone: #3 int,
            };
        "#},
        indoc! {r#"
            version v2;

            Contact = #1 enum {
                email: #2 string,
                phone: #3 string,
            };
        "#},
    );
}

#[test]
fn enum_remove_variant() {
    check(
        indoc! {r#"
            version v1;

            Contact = #1 enum {
                email: #2 string,
                phone: #3 int,
            };
        "#},
        indoc! {r#"
            version v2;

            Contact = #1 enum {
                email: #2 string,
            };
        "#},
    );
}

#[test]
fn enum_rename_variant() {
    check(
        indoc! {r#"
            version v1;

            Contact = #1 enum {
                email: #2 string,
                phone: #3 int,
            };
        "#},
        indoc! {r#"
            version v2;

            Contact = #1 enum {
                email: #2 string,
                phone_number: #3 int,
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

#[test]
fn change_boxedness_struct() {
    check(
        indoc! {r#"
            version v1;

            upper = #1 enum {
                a: #2 int,
            };

            lower = #3 struct {
                a: #4 int,
                b: #5 upper,
            };
        "#},
        indoc! {r#"
            version v2;

            upper = #1 enum {
                a: #2 int,
                b: lower,
            };

            lower = #3 struct {
                a: #4 int,
                b: #5 upper,
            };
        "#},
    );
}

#[test]
fn change_boxedness_enum() {
    check(
        indoc! {r#"
            version v1;

            upper = #1 struct {
                a: #2 int,
            };

            lower = #3 enum {
                a: #4 int,
                b: #5 upper,
            };
        "#},
        indoc! {r#"
            version v2;

            upper = #1 struct {
                a: #2 int,
                b: lower,
            };

            lower = #3 enum {
                a: #4 int,
                b: #5 upper,
            };
        "#},
    );
}

#[test]
fn change_boxedness_alias() {
    check(
        indoc! {r#"
            version v1;

            upper = #1 enum {
                a: #2 int,
            };

            lower = #3 upper;

            reference = #4 lower;
        "#},
        indoc! {r#"
            version v2;

            upper = #1 enum {
                a: #2 int,
                b: lower,
            };

            lower = #3 upper;

            reference = #4 lower;
        "#},
    );
}
