use assert_cmd::Command;
use predicates::prelude::{PredicateBooleanExt, predicate};
use std::path::Path;

/// Locate a JDK tool (`javac`, `jar`), preferring `$JAVA_HOME/bin`, falling
/// back to the system PATH — mirrors the binary's own `java_tool_command`.
/// Locate a JDK tool (`javac`, `jar`), preferring `$JAVA_HOME/bin`, then falling
/// back to PATH — mirroring the binary's own tool lookup.
fn jdk_tool(tool: &str) -> std::process::Command {
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        let mut candidate = Path::new(&home).join("bin").join(tool);
        if cfg!(windows) && candidate.extension().is_none() {
            candidate.set_extension("exe");
        }
        if candidate.is_file() {
            return std::process::Command::new(candidate);
        }
    }
    std::process::Command::new(tool)
}

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("local-annot-runtime")
}

/// Build the annotation library jar from the fixture's `lib-src` sources so it
/// can be referenced as a local dependency (`path=`) by the fixture project.
fn build_lib_jar(dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let lib_src = dest.join("lib-src");
    let classes = dest.join("jar-classes");
    std::fs::create_dir_all(&classes)?;
    std::fs::create_dir_all(dest.join("lib"))?;

    let java_src: Vec<_> = walkdir(&lib_src, "java")?;

    let status = jdk_tool("javac")
        .args(["-d", classes.to_str().unwrap()])
        .args(&java_src)
        .status()?;
    assert!(status.success(), "javac of the annotation library failed");

    // Include the processor service registration in the built jar so the
    // library is detected as an annotation processor.
    copy_dir(&lib_src.join("META-INF"), &classes.join("META-INF"))?;

    let jar = dest.join("lib").join("myannot.jar");
    let status = jdk_tool("jar")
        .current_dir(&classes)
        .args(["--create", "--file", jar.to_str().unwrap(), "."])
        .status()?;
    assert!(status.success(), "jar packaging failed");
    Ok(())
}

fn walkdir(dir: &Path, ext: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir(&path, ext)?);
        } else if path.extension().map_or(false, |e| e == ext) {
            out.push(path.to_string_lossy().to_string());
        }
    }
    Ok(out)
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        let dst = to.join(entry.file_name());
        if path.is_dir() {
            std::fs::create_dir_all(&dst)?;
            copy_dir(&path, &dst)?;
        } else {
            std::fs::copy(&path, &dst)?;
        }
    }
    Ok(())
}

#[test]
fn local_lib_jar_is_required_at_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    // Build the local annotation library jar referenced by lazy-java.toml.
    build_lib_jar(&dest)?;

    // Build — runs the annotation processor from the local jar, generating code.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    assert!(
        dest.join("target")
            .join("generated-source")
            .join("generated")
            .join("GeneratedHello.java")
            .exists(),
        "GeneratedHello.java should be produced by the local jar's processor"
    );
    assert!(
        dest.join("target").join("bin").join("app").join("Main.class").exists(),
        "Main.class should exist after build"
    );

    // Run — the program needs the annotation library's runtime class at runtime.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--no-build", "app.Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello-from-lib-annotations"))
        .stdout(predicate::str::contains("generated-by-processor"));

    // A second build with no changes must not trigger a full rebuild.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("full build").not());

    // Changing the local jar's contents forces a full rebuild.
    let greet = dest.join("lib-src").join("runtime").join("RuntimeHelper.java");
    let original_greet = std::fs::read_to_string(&greet)?;
    std::fs::write(&greet, original_greet.replace("hello-from-lib-annotations", "hello-2"))?;
    build_lib_jar(&dest)?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("full build"));

    Ok(())
}