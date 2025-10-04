#![allow(dead_code)]

use std::process::Command;

pub trait CommandExt {
    fn run_and_check(&mut self) -> String;
}

impl CommandExt for Command {
    fn run_and_check(&mut self) -> String {
        let output = self.output().unwrap();

        assert!(
            output.status.success(),
            "Error running {}:\n{}",
            self.get_program().to_string_lossy(),
            String::from_utf8_lossy(if output.stderr.is_empty() {
                &output.stdout
            } else {
                &output.stderr
            })
        );

        String::from_utf8(output.stdout).unwrap()
    }
}

pub const TSC_OPTIONS: &[&str] = &[
    "--noEmit", "--strict", "--lib", "es2022", "--target", "es2022",
];
