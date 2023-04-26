//! Tests for `cargo <file>.rs`

use cargo_test_support::project;

#[cargo_test]
fn permit_command() {
    let p = project()
        .file(
            "file.rs",
            r#"
#!/usr/bin/env cargo

fn main() {
    println!("Hello World");
}
            "#,
        )
        .build();

    p.cargo("-Z unstable-options file.rs")
        .masquerade_as_nightly_cargo(&["cargo-script"])
        .run();
}

#[cargo_test]
fn requires_nightly() {
    let p = project()
        .file(
            "file.rs",
            r#"
#!/usr/bin/env cargo

fn main() {
    println!("Hello World");
}
            "#,
        )
        .build();

    p.cargo("file.rs")
        .with_stderr(
            "\
[ERROR] the `cargo <file>.rs` command is unstable, and only available on the nightly channel of \
Cargo, but this is the `stable` channel
See [..] for more information about Rust release channels.
See [..] for more information about the `cargo <file>.rs` command.
",
        )
        .with_status(101)
        .run();
}

#[cargo_test]
fn requires_unstable_options() {
    let p = project()
        .file(
            "file.rs",
            r#"
#!/usr/bin/env cargo

fn main() {
    println!("Hello World");
}
            "#,
        )
        .build();

    p.cargo("file.rs")
        .masquerade_as_nightly_cargo(&["cargo-script"])
        .with_stderr(
            "\
[ERROR] the `cargo <file>.rs` command is unstable, pass `-Z unstable-options` to enable it
See [..] for more information about the `cargo <file>.rs` command.
",
        )
        .with_status(101)
        .run();
}
