//! Tests for nested workspaces
use cargo_test_support::project;

#[cargo_test]
fn permit_nested_simple() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        // Should not warn about unused fields.
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn permit_nested_detailed() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = { optional = true, path = "../" }
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        // Should not warn about unused fields.
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn deny_nested_false() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = false
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        // Should not warn about unused fields.
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  `false` is not an allowed value for key `workspace.nested`
",
        )
        .run();
}
