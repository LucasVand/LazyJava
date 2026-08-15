use predicates::prelude::{PredicateBooleanExt, predicate};

use crate::support::{build_lib_jar, lazy_java, sanitize_toml, Project};

#[test]
fn local_lib_jar_is_required_at_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let p = Project::from_fixture("local-annot-runtime")?;
    let dest = &p.dir;

    // Build the local annotation library jar referenced by lazy-java.toml.
    build_lib_jar(dest, "myannot.jar")?;

    // Build — runs the annotation processor from the local jar, generating code.
    lazy_java(dest)?.args(["build"]).assert().success();

    // The lock file records the local dependency jar with its detected
    // annotation processor configuration.
    let lock_path = dest.join("lazy-java.lock");
    assert!(
        lock_path.exists(),
        "lazy-java.lock should exist after build"
    );
    let lock_content = sanitize_toml(&std::fs::read_to_string(&lock_path)?, dest);
    insta::assert_snapshot!("local_lock", lock_content);

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
    lazy_java(dest)?
        .args(["run", "--no-build", "app.Main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello-from-lib-annotations"))
        .stdout(predicate::str::contains("generated-by-processor"));

    // A second build with no changes must not trigger a full rebuild.
    lazy_java(dest)?
        .args(["build"])
        .assert()
        .success()
        .stdout(predicate::str::contains("full build").not());

    // Changing the local jar's contents forces a full rebuild.
    let greet = dest.join("lib-src").join("runtime").join("RuntimeHelper.java");
    let original_greet = std::fs::read_to_string(&greet)?;
    std::fs::write(&greet, original_greet.replace("hello-from-lib-annotations", "hello-2"))?;
    build_lib_jar(dest, "myannot.jar")?;

    lazy_java(dest)?
        .args(["build"])
        .assert()
        .success()
        .stdout(predicate::str::contains("full build"));

    // Local dependencies have no CLI add/remove — they are managed by editing
    // the toml. Removing the entry and re-syncing must drop it from the lock.
    let config_path = dest.join("lazy-java.toml");
    let mut config_content = std::fs::read_to_string(&config_path)?;
    if let Some(idx) = config_content.find("[dependencies]") {
        config_content.truncate(idx);
    }
    std::fs::write(&config_path, &config_content)?;
    assert!(
        !config_content.contains("myannot"),
        "config should no longer reference the local jar after editing"
    );

    lazy_java(dest)?.args(["sync"]).assert().success();

    let lock_after_remove = sanitize_toml(&std::fs::read_to_string(&lock_path)?, dest);
    assert!(
        !lock_after_remove.contains("myannot"),
        "lock should no longer reference the local jar after sync"
    );
    insta::assert_snapshot!("local_lock_after_remove", lock_after_remove);

    Ok(())
}