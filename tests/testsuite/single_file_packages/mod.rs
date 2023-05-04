mod inner_block_comment;
mod no_extension;
mod permit_command;
mod requires_nightly;
mod requires_unstable_options;
mod script_with_deps;
mod shadows_run;
mod shadows_run_path_components_priority;

fn init_registry() {
    cargo_test_support::registry::init();
    add_registry_packages(false);
}

fn add_registry_packages(alt: bool) {
    cargo_test_support::registry::Package::new("baz", "1.0.0")
        .file(
            "src/lib.rs",
            "pub fn hello_world() { println!(\"Hello world!\"); }",
        )
        .alternative(alt)
        .publish();
}
