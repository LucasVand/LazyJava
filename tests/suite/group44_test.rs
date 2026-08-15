use crate::support::{normalize_output, run, Project};

#[test]
fn group44_full_build_graph_and_incremental_snapshots() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("group44")?;
    let dest = &p.dir;

    // Baseline: full build
    insta::assert_snapshot!(
        "full_build_output",
        normalize_output(&run(dest, &["build", "--show-compiled"])?, dest)
    );

    // Dependency and dependents graphs
    insta::assert_snapshot!(
        "dependency_graph",
        normalize_output(&run(dest, &["build", "dependencies"])?, dest)
    );
    insta::assert_snapshot!(
        "dependents_graph",
        normalize_output(&run(dest, &["build", "dependents"])?, dest)
    );

    // Nothing should be stale right after a build
    insta::assert_snapshot!(
        "stale_after_full_build",
        normalize_output(&run(dest, &["build", "stale"])?, dest)
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
        normalize_output(&run(dest, &["build", "stale"])?, dest)
    );

    // The incremental build must recompile the edited file plus all its
    // transitive dependents, and still succeed.
    insta::assert_snapshot!(
        "incremental_build_after_color_manager_edit",
        normalize_output(&run(dest, &["build", "--show-compiled"])?, dest)
    );

    // After the incremental build nothing should be stale.
    insta::assert_snapshot!(
        "stale_after_incremental_build",
        normalize_output(&run(dest, &["build", "stale"])?, dest)
    );

    // A repeated build with no further changes skips compilation entirely.
    insta::assert_snapshot!(
        "incremental_build_no_changes",
        normalize_output(&run(dest, &["build"])?, dest)
    );

    Ok(())
}