use std::{env, fs, iter::repeat_n, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

mod utils;

const MANIFEST_CONTENT: &str = indoc! {r#"
    [package]
    name = "versed_fixture"
    version = "0.1.0"
    edition = "2024"

    [[bin]]
    name = "versed_fixture"
    path = "src/mod.rs"

    [dependencies]
    serde = { version = "1.0.219", features = ["derive"] }
    postcard = { version = "1.1.3", features = ["use-std"] }
"#};

fn check(schema: &str, r#type: &str, value: &str) {
    let dir = tempdir().unwrap();

    let manifest_path = dir.path().join("Cargo.toml");
    fs::write(&manifest_path, MANIFEST_CONTENT).unwrap();

    let src_path = dir.path().join("src");
    fs::create_dir(&src_path).unwrap();

    let mod_path = src_path.join("mod.rs");
    let content = format!(
        indoc! {r#"
            #[allow(unused_imports)]
            fn main() {{
                let before: {type} = {value};
                let payload = postcard::to_stdvec(&before).unwrap();
                let after: {type} = postcard::from_bytes(&payload).unwrap();
                assert_eq!(before, after);
            }}
        "#},
        value = value,
        r#type = r#type,
    );
    fs::write(&mod_path, content).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, schema).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg("--serde")
        .arg("--serde-external-tag")
        .arg("--derive")
        .arg("PartialEq")
        .arg(schema_path)
        .arg(src_path)
        .run_and_check();

    Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .run_and_check();
}

#[test]
fn r#struct() {
    check(
        indoc! {"
            version v1;

            Point = struct {
                x: int,
                y: int
            };
        "},
        "v1::Point",
        "v1::Point { x: 10, y: 20 }",
    );
}

#[test]
fn r#enum() {
    check(
        indoc! {"
            version v1;

            Color = enum {
                red: int,
                green: string,
                blue: unit,
                yellow
            };
        "},
        "v1::Color",
        "v1::Color::Green(\"dark green\".to_owned())",
    );
}

#[test]
fn empty_struct() {
    check(
        indoc! {"
            version v1;

            Nothing = struct {};
        "},
        "v1::Nothing",
        "v1::Nothing {}",
    );
}

#[test]
fn references() {
    check(
        indoc! {"
            version v1;

            User = struct {
                name: Name,
                gender: Gender,
            };

            Name = struct { first: string, second: string };
            Gender = enum { male, female, other: string };
        "},
        "v1::User",
        indoc! {"
            v1::User {
                name: v1::Name { first: \"Benjamin\".to_owned(), second: \"Swart\".to_owned() },
                gender: v1::Gender::Male(()),
            }
        "},
    );
}

#[test]
fn type_alias() {
    check(
        indoc! {"
            version v1;

            Name = string;
        "},
        "v1::Name",
        "\"Benjamin Swart\".to_owned()",
    );
}

#[test]
fn nested_structs_enums() {
    let mut schema = String::from("version v1; Type = ");
    let mut value = String::new();

    for i in 0..50 {
        schema.push_str("struct { a: enum { a: ");

        let name_as: String = repeat_n('A', i * 2).collect();
        value.push_str(&format!("v1::Type{name_as} {{ a: v1::Type{name_as}A::A("));
    }

    schema.push_str("int");
    value.push_str("1337");

    for _ in 0..50 {
        schema.push_str(" } }");
        value.push_str(") }");
    }

    schema.push_str(";");

    check(&schema, "v1::Type", &value)
}

#[test]
fn nested_arrays() {
    check(
        indoc! {"
            version v1;

            Array = [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[int]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]];
        "},
        "v1::Array",
        "vec![vec![vec![vec![vec![vec![vec![], vec![vec![], vec![]], vec![]]], vec![vec![vec![vec![]]]]]]]]",
    );
}

#[test]
fn recursive_with_list() {
    check(
        indoc! {"
            version v1;

            User = struct {
                subordinates: [User]
            };
        "},
        "v1::User",
        indoc! {"
            v1::User { subordinates: vec![
                v1::User { subordinates: vec![] },
                v1::User { subordinates: vec![v1::User { subordinates: vec![] }] },
                v1::User { subordinates: vec![] }
            ] }
        "},
    );
}

#[test]
fn recursive_with_enum() {
    check(
        indoc! {"
            version v1;

            User = struct {
                admin: enum { some: User, none }
            };
        "},
        "v1::User",
        indoc! {"
            v1::User {
                admin: v1::UserAdmin::Some(Box::new(v1::User {
                    admin: v1::UserAdmin::None(())
                }))
            }
        "},
    );
}

#[test]
fn recursive_alias() {
    check(
        indoc! {"
            version v1;

            Set = [Set];
        "},
        "v1::Set",
        "v1::Set(vec![v1::Set(vec![]), v1::Set(vec![v1::Set(vec![])])])",
    );
}
