use std::fs;

use super::pom::import_pom;
use crate::args::ImportPomArgs;

#[test]
fn existing_toml_without_overwrite_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("pom.xml"), r#"<project>
        <groupId>com.example</groupId>
        <artifactId>test</artifactId>
        <version>1.0</version>
    </project>"#)
    .unwrap();

    fs::write(root.join("lazy-java.toml"), r#"[project]
name = "original""#)
    .unwrap();

    import_pom(
        root,
        &ImportPomArgs {
            pom_path: "pom.xml".into(),
            overwrite: false,
        },
    )
    .unwrap();

    let content = fs::read_to_string(root.join("lazy-java.toml")).unwrap();
    assert!(content.contains("original"), "should not overwrite");
}

#[test]
fn existing_toml_with_overwrite_flag_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("pom.xml"), r#"<project>
        <groupId>com.example</groupId>
        <artifactId>test</artifactId>
        <version>1.0</version>
    </project>"#)
    .unwrap();

    fs::write(root.join("lazy-java.toml"), r#"[project]
name = "original""#)
    .unwrap();

    import_pom(
        root,
        &ImportPomArgs {
            pom_path: "pom.xml".into(),
            overwrite: true,
        },
    )
    .unwrap();

    let content = fs::read_to_string(root.join("lazy-java.toml")).unwrap();
    assert!(content.contains("test"), "should overwrite with pom data");
}
