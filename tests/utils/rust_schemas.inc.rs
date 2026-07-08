#[test]
fn rust_type_idents() {
    check(indoc! {r#"
        version v1;

        String = unit;

        "" = struct {
            vec: struct {}
        };

        Option = struct {
            a: string,
            b: [int],
        };
    "#});
}

#[test]
fn rust_trait_idents() {
    check(indoc! {r#"
        version v1;

        Serialize = struct {
            Debug: int
        };

        Clone = enum {
            Copy: int
        };

        Eq = int;
    "#});
}

#[test]
fn rust_library_idents() {
    check(indoc! {r#"
        version v1;

        std = struct {
            core: int
        };

        serde = enum {
            serde: int
        };

        core = int;
    "#});
}

#[test]
fn rust_macro_idents() {
    check(indoc! {r#"
        version v1;

        assert = struct {
            assert: int
        };

        todo = enum {
            todo: int
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
