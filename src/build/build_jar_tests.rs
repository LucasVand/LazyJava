use std::{fs, path::Path};
use tempfile::tempdir;

use crate::build::build_jar::{build_manifest, merge_services};
use crate::config::ConfigTomlEdit;
use crate::Context;

fn test_ctx(target: &Path) -> Context {
    Context::new_options(
        None,
        Some(ConfigTomlEdit::parse(
            &format!(
                r#"[setup]
src = "src"
target = "{}"
"#,
                target.display()
            ),
        )
        .unwrap()),
    )
    .unwrap()
}

#[test]
fn build_manifest_empty_lib_no_classpath() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let target = tmp.path().join("target");
    let lib = target.join("lib");
    let lib_a = target.join("lib-annotations");
    fs::create_dir_all(&lib)?;
    fs::create_dir_all(&lib_a)?;
    // Create a dummy lazy-java.toml so the context's root is valid
    fs::write(tmp.path().join("lazy-java.toml"), b"")?;

    let ctx = test_ctx(&target);
    let manifest = build_manifest("com.example.Main", &ctx)?;

    assert!(manifest.contains("Manifest-Version: 1.0"));
    assert!(manifest.contains("Main-Class: com.example.Main"));
    assert!(!manifest.contains("Class-Path:"));
    Ok(())
}

#[test]
fn build_manifest_with_jars_includes_classpath() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let target = tmp.path().join("target");
    let lib = target.join("lib");
    let lib_a = target.join("lib-annotations");
    fs::create_dir_all(&lib)?;
    fs::create_dir_all(&lib_a)?;
    fs::write(tmp.path().join("lazy-java.toml"), b"")?;

    let jar_path = lib.join("example-1.0.jar");
    fs::write(&jar_path, b"dummy jar content")?;

    let ctx = test_ctx(&target);
    let manifest = build_manifest("Main", &ctx)?;

    assert!(manifest.contains("Class-Path:"));
    assert!(manifest.contains("lib/example-1.0.jar"));
    Ok(())
}

#[test]
fn merge_services_sorts_and_deduplicates() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let services = tmp.path().join("META-INF").join("services");
    fs::create_dir_all(&services)?;

    let svc = services.join("com.example.Service");
    fs::write(&svc, "z.example.Impl\nz.example.Impl\na.example.Impl\n")?;

    merge_services(tmp.path())?;

    let merged = fs::read_to_string(&svc)?;
    let lines: Vec<&str> = merged.lines().collect();
    assert_eq!(lines, vec!["a.example.Impl", "z.example.Impl"]);
    Ok(())
}

#[test]
fn merge_services_handles_missing_dir() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    merge_services(tmp.path())?;
    Ok(())
}

#[test]
fn merge_services_handles_multi_file() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let services = tmp.path().join("META-INF").join("services");
    fs::create_dir_all(&services)?;

    fs::write(services.join("a.Service"), "x.Impl\n")?;
    fs::write(services.join("b.Service"), "y.Impl\ny.Impl\n")?;

    merge_services(tmp.path())?;

    let a = fs::read_to_string(services.join("a.Service"))?;
    assert_eq!(a.lines().collect::<Vec<_>>(), vec!["x.Impl"]);

    let b = fs::read_to_string(services.join("b.Service"))?;
    assert_eq!(b.lines().collect::<Vec<_>>(), vec!["y.Impl"]);
    Ok(())
}
