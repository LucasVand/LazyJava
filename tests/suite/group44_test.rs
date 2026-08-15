use assert_cmd::Command;
use std::path::{Path, PathBuf};

use crate::support::normalize_output;

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("group44")
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
fn group44_full_build_graph_and_incremental_snapshots() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = copy_fixture();
    let dest = tmp.path().join("project");

    // Baseline: full build
    insta::assert_snapshot!(
        "full_build_output",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // Dependency and dependents graphs
    insta::assert_snapshot!(
        "dependency_graph",
        normalize_output(&run(&dest, &["build", "dependencies"])?, &dest)
    );
    insta::assert_snapshot!(
        "dependents_graph",
        normalize_output(&run(&dest, &["build", "dependents"])?, &dest)
    );

    // Nothing should be stale right after a build
    insta::assert_snapshot!(
        "stale_after_full_build",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // Edit a low-level utility that is imported across the whole UI,
    // expecting the incremental build to fan out to every transitive
    // dependents.
    let color_manager = dest.join("src/utils/ColorManager.java");
    let mut contents = std::fs::read_to_string(&color_manager)?;
    contents.push_str("\n    // modified by the group44 e2e test\n");
    std::fs::write(&color_manager, contents)?;

    insta::assert_snapshot!(
        "stale_after_color_manager_edit",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // The incremental build must recompile the edited file plus all its
    // transitive dependents, and still succeed.
    insta::assert_snapshot!(
        "incremental_build_after_color_manager_edit",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // After the incremental build nothing should be stale.
    insta::assert_snapshot!(
        "stale_after_incremental_build",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // A repeated build with no further changes skips compilation entirely.
    insta::assert_snapshot!(
        "incremental_build_no_changes",
        normalize_output(&run(&dest, &["build"])?, &dest)
    );

    Ok(())
}