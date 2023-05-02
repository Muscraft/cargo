use cargo_test_support::prelude::*;
use cargo_test_support::Project;

use super::init_registry;
use cargo_test_support::curr_dir;

#[cargo_test]
fn case() {
    init_registry();
    let project = Project::from_template(curr_dir!().join("in"));
    let project_root = project.root();
    let cwd = &project_root;

    snapbox::cmd::Command::cargo_ui()
        .arg("-Zunstable-options")
        .arg("file")
        .masquerade_as_nightly_cargo(&["cargo-script"])
        .current_dir(cwd)
        .assert()
        .success()
        .stdout_matches_path(curr_dir!().join("stdout.log"))
        .stderr_matches_path(curr_dir!().join("stderr.log"));
}
