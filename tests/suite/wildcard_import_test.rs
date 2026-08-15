use predicates::prelude::predicate;

use crate::support::{lazy_java, normalize_output, run, Project};

#[test]
fn wildcard_import_expands_to_all_files_in_package() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("wildcard-imports")?;
    let dest = &p.dir;

    // Baseline: full build
    insta::assert_snapshot!(
        "full_build_output",
        normalize_output(&run(dest, &["build", "--show-compiled"])?, dest)
    );

    // `Main` uses a wildcard import (`com.example.widgets.*`), so the graph must
    // record *all* widget files as its dependencies, not just the one it uses.
    insta::assert_snapshot!(
        "dependency_graph",
        normalize_output(&run(dest, &["build", "dependencies"])?, dest)
    );

    // Nothing should be stale right after a build
    insta::assert_snapshot!(
        "stale_after_full_build",
        normalize_output(&run(dest, &["build", "stale"])?, dest)
    );

    // Editing a widget file that `Main` never references directly must still fan
    // out to `Main`, because the wildcard import binds it to the whole package.
    std::fs::write(
        dest.join("src/widgets/WidgetB.java"),
        "package com.example.widgets;\n\npublic class WidgetB {\n    public static String render() {\n        return \"BETA\";\n    }\n}\n",
    )?;

    insta::assert_snapshot!(
        "stale_after_widget_b_edit",
        normalize_output(&run(dest, &["build", "stale"])?, dest)
    );

    // The incremental build must recompile the edited widget plus its wildcard
    // dependents (`Main`), and still succeed.
    insta::assert_snapshot!(
        "incremental_build_after_widget_b_edit",
        normalize_output(&run(dest, &["build", "--show-compiled"])?, dest)
    );

    // Functional check: the recompiled `Main` still produces working output.
    lazy_java(dest)?
        .args(["run", "--no-build", "com.example.Main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"));

    Ok(())
}