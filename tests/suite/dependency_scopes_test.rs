use assert_cmd::Command;
use predicates::prelude::{predicate, PredicateBooleanExt};
use std::path::Path;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("scope-dependency")
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

/// Replace the absolute temp project root with `<ROOT>` so lock snapshots are
/// deterministic across machines (canonical and symlinked forms).
fn sanitize_lock(content: &str, root: &std::path::Path) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    let canonical = std::fs::canonicalize(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| root.clone());

    let mut normalized = content.replace('\\', "/");
    if !canonical.is_empty() && canonical != root {
        normalized = normalized.replace(&canonical, "<ROOT>");
    }
    normalized.replace(&root, "<ROOT>")
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

    let lock = sanitize_lock(
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

    let lock = sanitize_lock(
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
