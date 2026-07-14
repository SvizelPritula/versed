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
    use std::iter::repeat_n;

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

#[test]
fn complex() {
    check(
        indoc! {"
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
        "},
        "v1::User",
        indoc! {r##"
            {
                use v1::*;
                User {
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
                }
            }
        "##},
    );
}
