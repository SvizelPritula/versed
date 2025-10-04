use std::{env, fs, process::Command};

use indoc::indoc;
use tempfile::tempdir;

use utils::CommandExt;

use crate::utils::TSC_OPTIONS;

mod utils;

const SCHEMA: &str = indoc! {"
    version v1;

    User = struct {
        first_name: string,
        last_name: string,

        role: Role,
        favourite_set: Set,
    };

    Role = enum {
        customer,
        store_employee: struct {
            supervisor: User,
        },
        supervisor: struct {
            branch: string,
        },
        admin,
    };

    Set = [Set];
"};

const MOD_CONTENT: &str = indoc! {r#"
    use v1::{Role, RoleStoreEmployee, RoleSupervisor, Set, User};
    
    fn main() {
        let user = User {
            first_name: "John".to_string(),
            last_name: "Watson".to_string(),

            role: Role::StoreEmployee(RoleStoreEmployee {
                supervisor: Box::new(User {
                    first_name: "Sherlock".to_string(),
                    last_name: "Holmes".to_string(),

                    role: Role::Supervisor(RoleSupervisor {
                        branch: "221B Baker Street".to_string(),
                    }),
                    favourite_set: Set(vec![Set(vec![Set(vec![Set(vec![Set(vec![])])])])]),
                }),
            }),
            favourite_set: Set(vec![Set(vec![Set(vec![])]), Set(vec![])]),
        };

        let content = serde_json::to_string_pretty(&user).unwrap();
        println!("{}", content);
    }
"#};

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
    serde_json = "1.0.143"
"#};

#[test]
fn roundtrip() {
    let dir = tempdir().unwrap();

    let manifest_path = dir.path().join("Cargo.toml");
    fs::write(&manifest_path, MANIFEST_CONTENT).unwrap();

    let src_path = dir.path().join("src");
    fs::create_dir(&src_path).unwrap();

    let mod_path = src_path.join("mod.rs");
    fs::write(&mod_path, MOD_CONTENT).unwrap();

    let schema_path = dir.path().join("schema.vd");
    fs::write(&schema_path, SCHEMA).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("rust")
        .arg("types")
        .arg("--serde")
        .arg(&schema_path)
        .arg(&src_path)
        .run_and_check();

    let json = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .run_and_check();

    let typescript_path = dir.path().join("typescript");
    fs::create_dir(&typescript_path).unwrap();

    let index_path = typescript_path.join("index.ts");
    fs::write(&index_path, format!("let user: v1.User = {json};\n")).unwrap();

    Command::new(env!("CARGO_BIN_EXE_versed"))
        .arg("typescript")
        .arg("types")
        .arg(&schema_path)
        .arg(&typescript_path)
        .run_and_check();

    Command::new("tsc")
        .args(TSC_OPTIONS)
        .arg(index_path)
        .run_and_check();
}
