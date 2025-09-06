#[test]
fn readme() {
    const CONTENTS: &str = include_str!("../../README.md");

    let mut in_code_block = false;
    let mut body = String::new();
    let mut is_other_language = false;
    let mut contains_version = false;

    for line in CONTENTS.lines() {
        if line.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                body = String::new();
                is_other_language = line.len() > 3;
                contains_version = false;
            } else {
                if !is_other_language {
                    if !contains_version {
                        body.insert_str(0, "version v1;\n");
                    }

                    check(&body);
                }

                in_code_block = false;
            }
        } else {
            if in_code_block {
                body.push_str(line);
                body.push('\n');

                if line.starts_with("version ") {
                    contains_version = true;
                }
            }
        }
    }
}

#[test]
fn empty() {
    check(indoc! {"
        version v1;
    "});
}

#[test]
fn r#struct() {
    check(indoc! {"
        version v1;

        Point = struct {
            x: int,
            y: int
        };
    "});
}

#[test]
fn r#enum() {
    check(indoc! {"
        version v1;

        Color = enum {
            red: int,
            green: string,
            blue: unit
        };
    "});
}

#[test]
fn empty_struct() {
    check(indoc! {"
        version v1;

        Nothing = struct {};
    "});
}

#[test]
fn empty_enum() {
    check(indoc! {"
        version v1;

        Impossible = enum {};
    "});
}

#[test]
fn nested_structs_enums() {
    let mut schema = String::from("version v1; Type = ");

    for _ in 0..50 {
        schema.push_str("struct { a: enum { a: ");
    }

    schema.push_str("int");

    for _ in 0..50 {
        schema.push_str(" } }");
    }

    schema.push_str(";");

    check(&schema)
}

#[test]
fn references() {
    check(indoc! {"
        version v1;

        User = struct {
            name: Name,
            gender: Gender,
        };

        Name = struct { first: string, second: string };
        Gender = enum { male, female, other: string };
    "});
}

#[test]
fn type_alias() {
    check(indoc! {"
        version v1;

        Name = string;
    "});
}

#[test]
fn nested_arrays() {
    check(indoc! {"
        version v1;

        Array = [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[int]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]];
    "});
}

#[test]
fn bad_chars_in_name() {
    check(indoc! {r#"
        version v1;

        "" = struct {
            "\"": int,
            "\n": int,
            ".": int,
            "name-name": int,
        };
    "#});
}

#[test]
fn bad_start_char() {
    check(indoc! {"
        version v1;

        \"10\" = struct {
            \"1\": int,
            \"2\": int,
        };
    "});
}

#[test]
fn similar_names() {
    check(indoc! {"
        version v1;

        a = struct {
            a_b: int,
            A_b: int,
            a_B: int,
            A_B: int,
        };

        A = enum {
            a_b: int,
            A_b: int,
            a_B: int,
            A_B: int,
        };
    "});
}

#[test]
fn types_named_like_path() {
    check(indoc! {"
        version v1;

        User = struct {
            contact: struct {
                email: string
            }
        };

        UserContact = struct {
            email: int
        };
    "});
}

#[test]
fn recursive_with_list() {
    check(indoc! {"
        version v1;

        User = struct {
            subordinates: [User]
        };
    "});
}

#[test]
fn recursive_with_enum() {
    check(indoc! {"
        version v1;

        User = struct {
            admin: enum { some: User, none }
        };
    "});
}

#[test]
fn mutually_recursive() {
    check(indoc! {"
        version v1;

        A = struct { b: B };
        B = struct { c: C };
        C = struct { d: D };
        D = struct { a: A };
    "});
}

#[test]
fn recursive_alias() {
    check(indoc! {"
        version v1;

        A = [A];
        B = struct { a: A };
    "});
}

#[test]
fn self_alias() {
    check(indoc! {"
        version v1;

        A = A;
    "});
}
