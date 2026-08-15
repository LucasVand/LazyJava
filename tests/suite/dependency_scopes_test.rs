use assert_cmd::Command;
use predicates::prelude::{predicate, PredicateBooleanExt};
use std::path::Path;

use crate::support::sanitize_toml;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("scope-dependency")
}

fn with_dependency_fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("with-dependency")
}

/// Locate a JDK tool, preferring `$JAVA_HOME/bin` and falling back to PATH —
/// mirrors the binary's own tool lookup.
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

/// Build the local library jar referenced by the fixture's toml from the
/// fixture's `lib-src` sources so it can be referenced as a local dependency.
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
    assert!(status.success(), "javac of the local library failed");

    let jar = dest.join("lib").join("scopelib.jar");
    let status = jdk_tool("jar")
        .current_dir(&classes)
        .args(["--create", "--file", jar.to_str().unwrap(), "."])
        .status()?;
    assert!(status.success(), "jar packaging failed");
    Ok(())
}

fn walkdir(dir: &std::path::Path, ext: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(walkdir(&path, ext)?);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path.to_string_lossy().to_string());
        }
    }
    Ok(out)
}

/// A `scope = "provided"` local dependency must be on the compileclasspath
/// (the project compiles against it) but absent from the runtime classpath (the
/// compiled `Main` fails with `NoClassDefFoundError` when the JVM loads it).
#[test]
fn provided_scoped_dependency_missing_from_runtime_classpath(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    build_lib_jar(&dest)?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Compiling using full build"));

    // provided is on the compile classpath, so the direct reference resolves.
    assert!(
        dest.join("target")
            .join("bin")
            .join("app")
            .join("Main.class")
            .exists(),
        "Main.class should exist on the compile classpath"
    );
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("full build").not());

    // provided is excluded from the runtime classpath, so running fails.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--no-build", "app.Main"]);
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("NoClassDefFoundError"));

    let lock = sanitize_toml(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        &dest,
    );
    insta::assert_snapshot!("provided_scope_lock", lock);

    Ok(())
}

/// A `scope = "runtime"` dependency must NOT be on the compile classpath: a
/// source file that references the dependency's classes fails to compile.
#[test]
fn runtime_scoped_dependency_not_available_at_compile_time(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    build_lib_jar(&dest)?;

    let config_path = dest.join("lazy-java.toml");
    let config = std::fs::read_to_string(&config_path)?;
    std::fs::write(
        &config_path,
        config.replace("scope = \"provided\"", "scope = \"runtime\""),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));

    let lock = sanitize_toml(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        &dest,
    );
    insta::assert_snapshot!("runtime_scope_lock", lock);

    Ok(())
}

/// A `scope = "runtime"` dependency must be on the runtime classpath. The
/// fixture's `Main` loads `com.example.scopelib.Helper` reflectively (so it
/// does not need the dependency at compile time); the reflectively-called
/// method proves the jar is present when the program runs.
#[test]
fn runtime_scoped_dependency_available_at_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    build_lib_jar(&dest)?;

    let config_path = dest.join("lazy-java.toml");
    let config = std::fs::read_to_string(&config_path)?;
    std::fs::write(
        &config_path,
        config.replace("scope = \"provided\"", "scope = \"runtime\""),
    )?;

    let main = dest.join("src").join("app").join("Main.java");
    std::fs::write(
        &main,
        "package app;\n\
         public class Main {\n\
         \x20   public static void main(String[] args) throws Exception {\n\
         \x20       Class<?> helper = Class.forName(\"com.example.scopelib.Helper\");\n\
         \x20       System.out.println(helper.getMethod(\"greet\").invoke(null));\n\
         \x20   }\n\
         }\n",
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["run", "--no-build", "app.Main"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello-from-scope-lib"));

    Ok(())
}

/// List the entries inside a jar file via the JDK's `jar tf`.
fn jar_entries(jar: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let out = jdk_tool("jar")
        .args(["tf", jar.to_str().unwrap()])
        .output()?;
    assert!(out.status.success(), "jar tf failed");
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// `provided`-scoped dependencies are compile-time only: they must be absent
/// from both the plain jar's `Class-Path` manifest and the bundled fat jar,
/// while `compile`-scoped dependencies stay on both.
#[test]
fn provided_scoped_dependency_excluded_from_jars() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        with_dependency_fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-lang3", "3.12.0", "provided"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-collections4", "4.4", "compile"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build"]);
    cmd.assert().success();

    // Plain jar: `Class-Path` must list the compile-scoped jar only.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build", "jar", "--entry-point", "Main"]);
    cmd.assert().success();

    let plain_jar = dest.join("target").join("build.jar");
    let extract = dest.join("plain-extract");
    std::fs::create_dir_all(&extract)?;
    assert!(
        jdk_tool("jar")
            .current_dir(&extract)
            .args(["xf", plain_jar.to_str().unwrap()])
            .status()?
            .success(),
        "extracting the plain jar failed"
    );
    let manifest = std::fs::read_to_string(extract.join("META-INF").join("MANIFEST.MF"))?;
    assert!(
        manifest.contains("commons-collections4"),
        "Class-Path should include compile-scoped jar, got:\n{manifest}"
    );
    assert!(
        !manifest.contains("commons-lang3"),
        "Class-Path must not include provided-scoped jar, got:\n{manifest}"
    );

    // Fat jar: only compile-scoped classes are bundled.
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["build", "jar", "--entry-point", "Main", "--fat"]);
    cmd.assert().success();

    let entries = jar_entries(&dest.join("target").join("build.jar"))?;
    assert!(
        entries.contains("org/apache/commons/collections4/"),
        "fat jar should bundle compile-scoped classes, got:\n{entries}"
    );
    assert!(
        !entries.contains("org/apache/commons/lang3/"),
        "fat jar must not bundle provided-scoped classes, got:\n{entries}"
    );

    Ok(())
}

/// The `add` command's scope argument must persist into both `lazy-java.toml`
/// and the lock file's root package so `generate pom` can emit it.
#[test]
fn added_remote_dependency_scope_persists_to_config_and_lock(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    build_lib_jar(&dest)?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-lang3", "3.12.0", "runtime"]);
    cmd.assert().success();

    let toml = std::fs::read_to_string(dest.join("lazy-java.toml"))?;
    assert!(
        toml.contains("commons-lang3 = { version = \"3.12.0\", group = \"org.apache.commons\", scope = \"runtime\" }"),
        "config should record the runtime scope, got:\n{toml}"
    );

    let lock = std::fs::read_to_string(dest.join("lazy-java.lock"))?;
    assert!(
        lock.contains("scope = \"runtime\""),
        "lock should record the runtime scope, got:\n{lock}"
    );

    Ok(())
}

/// Editing a remote dependency's scope in `lazy-java.toml` must refresh the
/// lock on the next sync — not just for newly added deps.
#[test]
fn editing_scope_in_config_refreshes_lock() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        with_dependency_fixture_path(),
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["add", "org.apache.commons", "commons-lang3", "3.12.0", "provided"]);
    cmd.assert().success();

    let lock = std::fs::read_to_string(dest.join("lazy-java.lock"))?;
    assert!(
        lock.contains("scope = \"provided\""),
        "lock should record the provided scope, got:\n{lock}"
    );

    let config_path = dest.join("lazy-java.toml");
    let config = std::fs::read_to_string(&config_path)?;
    std::fs::write(
        &config_path,
        config.replace("scope = \"provided\"", "scope = \"runtime\""),
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["sync"]);
    cmd.assert().success();

    let lock = std::fs::read_to_string(dest.join("lazy-java.lock"))?;
    assert!(
        lock.contains("scope = \"runtime\"") && !lock.contains("scope = \"provided\""),
        "lock scope should be refreshed to runtime, got:\n{lock}"
    );

    Ok(())
}

/// Removing multiple local dependencies in a single sync must not corrupt the
/// lock file.
#[test]
fn removing_multiple_local_dependencies_in_one_sync() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    std::fs::create_dir_all(dest.join("lib"))?;
    std::fs::create_dir_all(dest.join("src"))?;

    // Build two minimal jars to use as local dependencies.
    let classes_a = dest.join("classes-a");
    let classes_b = dest.join("classes-b");
    std::fs::create_dir_all(&classes_a)?;
    std::fs::create_dir_all(&classes_b)?;
    let src_a = dest.join("A.java");
    let src_b = dest.join("B.java");
    std::fs::write(&src_a, "public class A {}\n")?;
    std::fs::write(&src_b, "public class B {}\n")?;

    let mut javac = jdk_tool("javac");
    assert!(
        javac
            .args(["-d", classes_a.to_str().unwrap()])
            .arg(src_a)
            .status()?
            .success(),
        "javac of the first local library failed"
    );
    let mut javac = jdk_tool("javac");
    assert!(
        javac
            .args(["-d", classes_b.to_str().unwrap()])
            .arg(src_b)
            .status()?
            .success(),
        "javac of the second local library failed"
    );

    let jar_a = dest.join("lib").join("aja.jar");
    let jar_b = dest.join("lib").join("bjb.jar");
    assert!(
        jdk_tool("jar")
            .current_dir(&classes_a)
            .args(["--create", "--file", jar_a.to_str().unwrap(), "."])
            .status()?
            .success(),
        "jar packaging of the first local library failed"
    );
    assert!(
        jdk_tool("jar")
            .current_dir(&classes_b)
            .args(["--create", "--file", jar_b.to_str().unwrap(), "."])
            .status()?
            .success(),
        "jar packaging of the second local library failed"
    );

    std::fs::write(
        dest.join("lazy-java.toml"),
        "[project]\n\
         name = \"multi-local\"\n\
         \n\
         [dependencies]\n\
         a = { path = \"lib/aja.jar\", scope = \"compile\" }\n\
         b = { path = \"lib/bjb.jar\", scope = \"compile\" }\n",
    )?;

    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["sync"]);
    cmd.assert().success();

    let lock = std::fs::read_to_string(dest.join("lazy-java.lock"))?;
    assert!(
        lock.contains("package-local"),
        "lock should contain the local packages, got:\n{lock}"
    );

    // Remove both local deps and re-sync.
    std::fs::write(
        dest.join("lazy-java.toml"),
        "[project]\nname = \"multi-local\"\n",
    )?;
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(&dest);
    cmd.args(["sync"]);
    cmd.assert().success();

    let lock = std::fs::read_to_string(dest.join("lazy-java.lock"))?;
    assert!(
        !lock.contains("package-local"),
        "lock should no longer contain local packages, got:\n{lock}"
    );

    Ok(())
}
