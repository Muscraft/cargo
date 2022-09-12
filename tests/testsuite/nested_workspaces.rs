//! Tests for nested workspaces
use cargo_test_support::project;

#[cargo_test]
fn permit_nested_simple() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        // Should not warn about unused fields.
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
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
            members = ["nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = { optional = true, path = "../" }
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        // Should not warn about unused fields.
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn crate_specifies_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
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
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  `false` is not an allowed value for key `workspace.nested`
",
        )
        .run();
}

#[cargo_test]
fn error_no_parent_exists() {
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
              "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  workspace at [CWD]/Cargo.toml is supposed to be nested but no parent workspace could be found
",
        )
        .run();
}

#[cargo_test]
fn error_nested_excluded_explicit_ws() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested/bar"]
            exclude = ["nested"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/nested/Cargo.toml`

Caused by:
  workspace at [CWD]/nested/Cargo.toml is supposed to be nested but no parent workspace could be found
",
        )
        .run();
}

#[cargo_test]
fn nested_ws_inherit() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested", "nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
            [workspace.package]
            version = "0.1.0"
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version.workspace = true
              authors = []
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn nested_ws_inherit_own() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested", "nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
            [workspace.package]
            version = "0.1.0"
            [package]
            name = "nested"
            version.workspace = true
            authors = []
        "#,
        )
        .file("nested/src/main.rs", "fn main() {}")
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version.workspace = true
              authors = []
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
[COMPILING] nested v0.1.0 ([CWD]/nested)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn nested_ws_inherit2() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
            [workspace.package]
            version = "0.1.0"
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version.workspace = true
              authors = []
              workspace = ".."
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn nested_ws_inherit_lowest_level() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested", "nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
            [workspace.package]
            version = "0.1.0"
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version.workspace = true
              authors = []
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .arg("--manifest-path")
        .arg(p.root().join("nested").join("bar").join("Cargo.toml"))
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn nested_ws_inherit_lowest_level2() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["nested/bar"]
        "#,
        )
        .file(
            "nested/Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            nested = true
            [workspace.package]
            version = "0.1.0"
        "#,
        )
        .file(
            "nested/bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version.workspace = true
              authors = []
              workspace = ".."
              "#,
        )
        .file("nested/bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .arg("--manifest-path")
        .arg(p.root().join("nested").join("bar").join("Cargo.toml"))
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/nested/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}
