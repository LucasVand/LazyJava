use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::{Path, PathBuf};

use crate::support::normalize_output;

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("wildcard-imports")
}

fn copy_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .unwrap();
    tmp
}

fn run(dest: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(dest);
    cmd.args(args);
    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "`lazy-java {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn wildcard_import_expands_to_all_files_in_package() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture();
    let dest = tmp.path().join("project");

    // Baseline: full build
    insta::assert_snapshot!(
        "full_build_output",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // `Main` uses a wildcard import (`com.example.widgets.*`), so the graph must
    // record *all* widget files as its dependencies, not just the one it uses.
    insta::assert_snapshot!(
        "dependency_graph",
        normalize_output(&run(&dest, &["build", "dependencies"])?, &dest)
    );

    // Nothing should be stale right after a build
    insta::assert_snapshot!(
        "stale_after_full_build",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // Editing a widget file that `Main` never references directly must still fan
    // out to `Main`, because the wildcard import binds it to the whole package.
    std::fs::write(
        dest.join("src/widgets/WidgetB.java"),
        "package com.example.widgets;\n\npublic class WidgetB {\n    public static String render() {\n        return \"BETA\";\n    }\n}\n",
    )?;

    insta::assert_snapshot!(
        "stale_after_widget_b_edit",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // The incremental build must recompile the edited widget plus its wildcard
    // dependents (`Main`), and still succeed.
    insta::assert_snapshot!(
        "incremental_build_after_widget_b_edit",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // Functional check: the recompiled `Main` still produces working output.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--no-build", "com.example.Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("alpha"));

    Ok(())
}