//! Tests for hints.

use crate::prelude::*;
use cargo_test_support::registry::Package;
use cargo_test_support::{project, str};

#[cargo_test]
fn empty_hints_no_warn() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [hints]
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -v")
        .with_stderr_data(str![[r#"
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .run();
}

#[cargo_test]
fn unknown_hints_warn() {
    Package::new("bar", "1.0.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "1.0.0"
            edition = "2015"

            [hints]
            this-is-an-unknown-hint = true
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [dependencies]
            bar = "1.0"

            [hints]
            this-is-an-unknown-hint = true
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -v")
        .with_stderr_data(str![[r#"
[WARNING] unused manifest key: hints.this-is-an-unknown-hint
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.0.0 (registry `dummy-registry`)
[CHECKING] bar v1.0.0
[RUNNING] `rustc --crate-name bar [..]`
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .run();
}

#[cargo_test]
fn hint_unknown_type_warn() {
    Package::new("bar", "1.0.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "1.0.0"
            edition = "2015"

            [hints]
            mostly-unused = 1
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [dependencies]
            bar = "1.0"

            [hints]
            mostly-unused = "string"
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -v")
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.0.0 (registry `dummy-registry`)
[WARNING] foo@0.0.1: ignoring unsupported value type (string) for 'hints.mostly-unused', which expects a boolean
[CHECKING] bar v1.0.0
[RUNNING] `rustc --crate-name bar [..]`
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .with_stderr_does_not_contain("-Zhint-mostly-unused")
        .run();
}

#[cargo_test]
fn hints_mostly_unused_warn_without_gate() {
    Package::new("bar", "1.0.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "1.0.0"
            edition = "2015"

            [hints]
            mostly-unused = true
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [dependencies]
            bar = "1.0"

            [hints]
            mostly-unused = true
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -v")
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.0.0 (registry `dummy-registry`)
[WARNING] foo@0.0.1: ignoring 'hints.mostly-unused', pass `-Zprofile-hint-mostly-unused` to enable it
[CHECKING] bar v1.0.0
[RUNNING] `rustc --crate-name bar [..]`
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .with_stderr_does_not_contain("-Zhint-mostly-unused")
        .run();
}

#[cargo_test(nightly, reason = "-Zhint-mostly-unused is unstable")]
fn hints_mostly_unused_nightly() {
    Package::new("bar", "1.0.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "1.0.0"
            edition = "2015"

            [hints]
            mostly-unused = true
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [dependencies]
            bar = "1.0"
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -Zprofile-hint-mostly-unused -v")
        .masquerade_as_nightly_cargo(&["profile-hint-mostly-unused"])
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.0.0 (registry `dummy-registry`)
[CHECKING] bar v1.0.0
[RUNNING] `rustc --crate-name bar [..] -Zhint-mostly-unused [..]`
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .with_stderr_does_not_contain(
            "[RUNNING] `rustc --crate-name foo [..] -Zhint-mostly-unused [..]",
        )
        .run();
}

#[cargo_test(nightly, reason = "-Zhint-mostly-unused is unstable")]
fn mostly_unused_profile_overrides_hints_nightly() {
    Package::new("bar", "1.0.0")
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "1.0.0"
            edition = "2015"

            [hints]
            mostly-unused = true
            "#,
        )
        .file("src/lib.rs", "")
        .publish();
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [dependencies]
            bar = "1.0"

            [profile.dev.package.bar]
            hint-mostly-unused = false
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -Zprofile-hint-mostly-unused -v")
        .masquerade_as_nightly_cargo(&["profile-hint-mostly-unused"])
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index
[LOCKING] 1 package to latest compatible version
[DOWNLOADING] crates ...
[DOWNLOADED] bar v1.0.0 (registry `dummy-registry`)
[CHECKING] bar v1.0.0
[RUNNING] `rustc --crate-name bar [..]`
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .with_stderr_does_not_contain("-Zhint-mostly-unused")
        .run();
}

#[cargo_test(nightly, reason = "-Zhint-mostly-unused is unstable")]
fn mostly_unused_profile_overrides_hints_on_self_nightly() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [hints]
            mostly-unused = true

            [profile.dev]
            hint-mostly-unused = false
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -v")
        .with_stderr_data(str![[r#"
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .with_stderr_does_not_contain("-Zhint-mostly-unused")
        .run();
}

#[cargo_test(nightly, reason = "-Zhint-mostly-unused is unstable")]
fn lint_global_mostly_unused() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [profile.dev]
            hint-mostly-unused = true
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -Zprofile-hint-mostly-unused -v")
        .masquerade_as_nightly_cargo(&["profile-hint-mostly-unused", "cargo-lints"])
        .with_stderr_data(str![[r#"
[WARNING] use of `hint-mostly-unused` in a global context
 --> Cargo.toml:8:13
  |
7 |             [profile.dev]
  |                      ---
8 |             hint-mostly-unused = true
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = [NOTE] `cargo::global_mostly_unused` is set to `warn` by default
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .run();
}

#[cargo_test(nightly, reason = "-Zhint-mostly-unused is unstable")]
fn lint_global_mostly_unused_all_pkg_spec() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            edition = "2015"

            [profile.dev.package."*"]
            hint-mostly-unused = true
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();
    p.cargo("check -Zprofile-hint-mostly-unused -v")
        .masquerade_as_nightly_cargo(&["profile-hint-mostly-unused", "cargo-lints"])
        .with_stderr_data(str![[r#"
[WARNING] use of `hint-mostly-unused` in a global context
 --> Cargo.toml:8:13
  |
7 |             [profile.dev.package."*"]
  |                                  ---
8 |             hint-mostly-unused = true
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = [NOTE] `cargo::global_mostly_unused` is set to `warn` by default
[CHECKING] foo v0.0.1 ([ROOT]/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .run();
}

#[cargo_test(nightly, reason = "-Zhint-mostly-unused is unstable")]
fn lint_global_mostly_unused_workspace() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
[workspace]
members = ["foo"]

[profile.dev.package."*"]
hint-mostly-unused = true
"#,
        )
        .file(
            "foo/Cargo.toml",
            r#"
[package]
name = "foo"
version = "0.0.1"
edition = "2015"
authors = []
            "#,
        )
        .file("foo/src/lib.rs", "")
        .build();

    p.cargo("check -Zprofile-hint-mostly-unused -v")
        .masquerade_as_nightly_cargo(&["profile-hint-mostly-unused", "cargo-lints"])
        .with_stderr_data(str![[r#"
[WARNING] use of `hint-mostly-unused` in a global context
 --> Cargo.toml:6:1
  |
5 | [profile.dev.package."*"]
  |                      ---
6 | hint-mostly-unused = true
  | ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = [NOTE] `cargo::global_mostly_unused` is set to `warn` by default
[CHECKING] foo v0.0.1 ([ROOT]/foo/foo)
[RUNNING] `rustc --crate-name foo [..]`
[FINISHED] `dev` profile [unoptimized + debuginfo] target(s) in [ELAPSED]s

"#]])
        .run();
}
