use predicates::prelude::predicate;
use predicates::prelude::PredicateBooleanExt;

use crate::support::{build_lib_jar, jdk_tool, lazy_java, sanitize_toml, Project};

fn jar_entries(jar: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let out = jdk_tool("jar")
        .args(["tf", jar.to_str().unwrap()])
        .output()?;
    assert!(out.status.success(), "jar tf failed");
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// A `scope = "provided"` local dependency must be on the compileclasspath
/// (the project compiles against it) but absent from the runtime classpath (the
/// compiled `Main` fails with `NoClassDefFoundError` when the JVM loads it).
#[test]
fn provided_scoped_dependency_missing_from_runtime_classpath(
) -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("scope-dependency")?;
    let dest = &p.dir;
    build_lib_jar(dest, "scopelib.jar")?;

    lazy_java(dest)?
        .args(["build"])
        .assert()
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
    lazy_java(dest)?
        .args(["build"])
        .assert()
        .success()
        .stdout(predicate::str::contains("full build").not());

    // provided is excluded from the runtime classpath, so running fails.
    lazy_java(dest)?
        .args(["run", "--no-build", "app.Main"])
        .assert()
        .success()
        .stderr(predicate::str::contains("NoClassDefFoundError"));

    let lock = sanitize_toml(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        dest,
    );
    insta::assert_snapshot!("provided_scope_lock", lock);

    Ok(())
}

/// A `scope = "runtime"` dependency must NOT be on the compile classpath: a
/// source file that references the dependency's classes fails to compile.
#[test]
fn runtime_scoped_dependency_not_available_at_compile_time(
) -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("scope-dependency")?;
    let dest = &p.dir;
    build_lib_jar(dest, "scopelib.jar")?;

    let config_path = dest.join("lazy-java.toml");
    let config = std::fs::read_to_string(&config_path)?;
    std::fs::write(
        &config_path,
        config.replace("scope = \"provided\"", "scope = \"runtime\""),
    )?;

    lazy_java(dest)?
        .args(["build"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));

    let lock = sanitize_toml(
        &std::fs::read_to_string(dest.join("lazy-java.lock"))?,
        dest,
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
    let p = Project::from_fixture("scope-dependency")?;
    let dest = &p.dir;
    build_lib_jar(dest, "scopelib.jar")?;

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

    lazy_java(dest)?.args(["build"]).assert().success();

    lazy_java(dest)?
        .args(["run", "--no-build", "app.Main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-from-scope-lib"));

    Ok(())
}

/// `provided`-scoped dependencies are compile-time only: they must be absent
/// from both the plain jar's `Class-Path` manifest and the bundled fat jar,
/// while `compile`-scoped dependencies stay on both.
#[test]
fn provided_scoped_dependency_excluded_from_jars() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("with-dependency")?;
    let dest = &p.dir;
    lazy_java(dest)?
        .args(["add", "org.apache.commons", "commons-lang3", "3.12.0", "provided"])
        .assert()
        .success();

    lazy_java(dest)?
        .args(["add", "org.apache.commons", "commons-collections4", "4.4", "compile"])
        .assert()
        .success();

    lazy_java(dest)?.args(["build"]).assert().success();

    // Plain jar: `Class-Path` must list the compile-scoped jar only.
    lazy_java(dest)?
        .args(["build", "jar", "--entry-point", "Main"])
        .assert()
        .success();

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
    lazy_java(dest)?
        .args(["build", "jar", "--entry-point", "Main", "--fat"])
        .assert()
        .success();

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
    let p = Project::from_fixture("scope-dependency")?;
    let dest = &p.dir;
    build_lib_jar(dest, "scopelib.jar")?;

    lazy_java(dest)?
        .args(["add", "org.apache.commons", "commons-lang3", "3.12.0", "runtime"])
        .assert()
        .success();

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
    let p = Project::from_fixture("with-dependency")?;
    let dest = &p.dir;
    lazy_java(dest)?
        .args(["add", "org.apache.commons", "commons-lang3", "3.12.0", "provided"])
        .assert()
        .success();

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

    lazy_java(dest)?.args(["sync"]).assert().success();

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
    let p = Project::empty()?;
    let dest = &p.dir;
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

    lazy_java(dest)?.args(["sync"]).assert().success();

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
    lazy_java(dest)?.args(["sync"]).assert().success();

    let lock = std::fs::read_to_string(dest.join("lazy-java.lock"))?;
    assert!(
        !lock.contains("package-local"),
        "lock should no longer contain local packages, got:\n{lock}"
    );

    Ok(())
}