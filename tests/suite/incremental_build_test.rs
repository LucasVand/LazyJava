use assert_cmd::Command;
use predicates::prelude::predicate;
use std::path::{Path, PathBuf};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("incremental-build")
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

/// Normalizes command output for deterministic snapshot comparison:
/// - absolute paths under the temp project dir become `<ROOT>`
/// - the non-deterministic `Compiled in X.XXs` line becomes `Compiled in <TIME>s`
/// - lines are sorted, since graph and HashSet iteration order is not stable
fn normalize_output(output: &str, root: &Path) -> String {
    // Normalize separators up front so the project root matches on both
    // Unix (`/`) and Windows (`\`) before substituting `<ROOT>`.
    let root_path = root.to_string_lossy().replace('\\', "/");
    let canonical_root = std::fs::canonicalize(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| root_path.clone());

    let mut lines: Vec<String> = output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let l = if l.starts_with("Compiled in") {
                "Compiled in <TIME>s"
            } else {
                l
            };
            let l = l.replace('\\', "/");
            // replace the canonical (symlink-resolved) path first so a
            // `/private/var/...` path does not leave a `/private` prefix
            l.replace(&canonical_root, "<ROOT>")
                .replace(&root_path, "<ROOT>")
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

const MODIFIED_FORMATTER: &str = r#"package com.example.greeting;

public class Formatter {
    public static String format(String name) {
        return name.toUpperCase() + "!";
    }
}
"#;

const MODIFIED_CALC: &str = r#"package com.example.math;

import com.example.math.Adder;
import com.example.math.Subtracter;

public class Calc {
    public static int add(int a, int b) {
        return Adder.add(a, b);
    }

    public static int subtract(int a, int b) {
        return Math.max(a, b);
    }
}
"#;

const MODIFIED_STRINGS: &str = r#"package com.example.util;

public class Strings {
    public static final String PREFIX = "static-import-after-edit ";

    public static String pad(String value) {
        return " " + value + " ";
    }
}
"#;

#[test]
fn incremental_build_dependency_and_subcommand_snapshots() -> Result<(), Box<dyn std::error::Error>>
{
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

    // Editing a leaf file (no dependents of its own) fans out to all transitive dependents
    std::fs::write(dest.join("src/greeting/Formatter.java"), MODIFIED_FORMATTER)?;

    insta::assert_snapshot!(
        "stale_after_leaf_edit",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );

    // Incremental build recompiles the file plus its transitive dependents
    insta::assert_snapshot!(
        "incremental_build_after_leaf_edit",
        normalize_output(&run(&dest, &["build", "--show-compiled"])?, &dest)
    );

    // A second build with no changes skips compilation entirely
    insta::assert_snapshot!(
        "incremental_build_no_changes",
        normalize_output(&run(&dest, &["build"])?, &dest)
    );

    // Editing a mid-graph file fans out to its same-package files and dependents
    std::fs::write(dest.join("src/math/Calc.java"), MODIFIED_CALC)?;

    insta::assert_snapshot!(
        "stale_after_mid_edit",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );
    insta::assert_snapshot!(
        "incremental_build_after_mid_edit",
        normalize_output(&run(&dest, &["build"])?, &dest)
    );

    // Editing a class consumed only through a `static` import must fan out to
    // the file that imports it (`Main`), proving static-import graph edges.
    std::fs::write(dest.join("src/util/Strings.java"), MODIFIED_STRINGS)?;

    insta::assert_snapshot!(
        "stale_after_strings_edit",
        normalize_output(&run(&dest, &["build", "stale"])?, &dest)
    );
    insta::assert_snapshot!(
        "incremental_build_after_strings_edit",
        normalize_output(&run(&dest, &["build"])?, &dest)
    );

    // Functional check: the incremental recompilation produced working output
    // reflecting both edits (Formatter appends "!", Calc.subtract is now Math.max)
    // and the static-imported constant from the edited Strings.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--no-build", "com.example.Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello, WORLD!"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("static-import-after-edit works"));

    Ok(())
}
