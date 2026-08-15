//! Shared helpers for the integration test suite.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// Copies a fixture directory into a fresh temp dir under `project/` and
/// returns the `TempDir`, so each test gets an isolated, writable copy of the
/// fixture without duplicating the `tempdir`+`fs_extra::dir::copy` boilerplate.
pub fn copy_fixture(fixture: &Path) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dest = tmp.path().join("project");
    fs_extra::dir::copy(
        fixture,
        &dest,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )?;
    Ok(tmp)
}

/// Absolute path to `tests/fixtures/<name>`.
pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// A writable, isolated copy of a project fixture plus its temporary root. Each
/// test gets fresh state, so fixtures are never mutated by tests.
pub struct Project {
    _tmp: tempfile::TempDir,
    /// The project root (`<tmp>/project`) where `lazy-java` commands run.
    pub dir: PathBuf,
}

impl Project {
    /// Creates a `project/` dir populated from `tests/fixtures/<name>`.
    pub fn from_fixture(name: &str) -> Result<Project, Box<dyn std::error::Error>> {
        let tmp = copy_fixture(&fixture_path(name))?;
        let dir = tmp.path().join("project");
        Ok(Project { _tmp: tmp, dir })
    }

    /// Creates an empty `project/` dir for tests that build the project up by
    /// hand instead of copying a fixture.
    pub fn empty() -> Result<Project, Box<dyn std::error::Error>> {
        let tmp = tempfile::tempdir()?;
        let dir = tmp.path().join("project");
        std::fs::create_dir_all(&dir)?;
        Ok(Project { _tmp: tmp, dir })
    }

    /// The temporary root that contains `dir`, for files that must sit outside
    /// the project root (e.g. external resources referenced via `../`).
    pub fn root(&self) -> &Path {
        self._tmp.path()
    }
}

/// An `assert_cmd` `lazy-java` command that runs in the given project dir.
pub fn lazy_java(dir: &Path) -> Result<Command, Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("lazy-java")?;
    cmd.current_dir(dir);
    Ok(cmd)
}

/// Runs `lazy-java` in `dir`, asserting it succeeds, and returns its stdout.
pub fn run(dir: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = lazy_java(dir)?.args(args).output()?;
    assert!(
        output.status.success(),
        "`lazy-java {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

/// Locate a JDK tool (`javac`, `jar`), preferring `$JAVA_HOME/bin` and falling
/// back to PATH — mirrors the binary's own tool lookup.
pub fn jdk_tool(tool: &str) -> std::process::Command {
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

/// Recursively collects the absolute paths of the `.java` files under `dir`.
pub fn collect_java_files(dir: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_java_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "java") {
            out.push(path.to_string_lossy().to_string());
        }
    }
    Ok(out)
}

/// Compiles the fixture's `lib-src` sources into `lib/<jar_name>` so the
/// project can reference them as a local `path=` dependency. A `META-INF/`
/// sibling (e.g. an annotation-processor service registration) is bundled into
/// the jar when present.
pub fn build_lib_jar(dest: &Path, jar_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lib_src = dest.join("lib-src");
    let classes = dest.join("jar-classes");
    std::fs::create_dir_all(&classes)?;
    std::fs::create_dir_all(dest.join("lib"))?;

    let java_src = collect_java_files(&lib_src)?;

    let status = jdk_tool("javac")
        .args(["-d", classes.to_str().unwrap()])
        .args(&java_src)
        .status()?;
    assert!(status.success(), "javac of the local library failed");

    let meta_inf = lib_src.join("META-INF");
    if meta_inf.is_dir() {
        fs_extra::dir::copy(
        &meta_inf,
        &classes,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )?;
    }

    let jar = dest.join("lib").join(jar_name);
    let status = jdk_tool("jar")
        .current_dir(&classes)
        .args(["--create", "--file", jar.to_str().unwrap(), "."])
        .status()?;
    assert!(status.success(), "jar packaging failed");
    Ok(())
}

/// Replaces the absolute project root with `<ROOT>` so lock/config snapshots
/// are deterministic across machines. The embedded paths can appear in either
/// canonical (`/private/var/...`) or symlinked (`/var/...`) form, so both are
/// substituted.
///
/// The toml serializer emits literal (single-quoted) strings when a path
/// contains backslashes (e.g. on Windows), so after substituting `<ROOT>` any
/// path lines that came out single-quoted are re-quoted back to the basic
/// double-quoted form used by the committed snapshots.
pub fn sanitize_toml(content: &str, root: &Path) -> String {
    let root = root.to_string_lossy().replace('\\', "/");
    let canonical = std::fs::canonicalize(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| root.clone());

    let mut normalized = content.replace('\\', "/");

    // Replace the canonical (symlink-resolved) path first so a `/private/var/...`
    // path does not leave a `/private` prefix behind.
    if !canonical.is_empty() && canonical != root {
        normalized = normalized.replace(&canonical, "<ROOT>");
    }
    normalized = normalized.replace(&root, "<ROOT>");

    // Literal (single-quoted) strings cannot contain a `"`, so a `<ROOT>` line
    // that has no double-quote must have come out single-quoted by the toml
    // serializer and is re-quoted here. Lines that already contain a `"` are
    // left untouched.
    normalized
        .lines()
        .map(|line| {
            if line.contains("<ROOT>") && !line.contains('"') {
                line.replace('\'', "\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Normalizes command output for deterministic snapshot comparison:
/// - absolute paths under the temp project dir become `<ROOT>`
/// - the non-deterministic `Compiled in X.XXs` line becomes `Compiled in <TIME>s`
/// - lines are sorted, since graph and HashSet iteration order is not stable
pub fn normalize_output(output: &str, root: &Path) -> String {
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
            // Replace the canonical (symlink-resolved) path first so a
            // `/private/var/...` path does not leave a `/private` prefix.
            l.replace(&canonical_root, "<ROOT>")
                .replace(&root_path, "<ROOT>")
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unix-style path serialized as a basic (double-quoted) toml string.
    #[test]
    fn sanitize_toml_substitutes_double_quoted_path() {
        let input = r#"[[package]]
path = "/var/folders/abc/T/proj/lib/scopelib.jar"
"#;
        let normalized = sanitize_toml(input, Path::new("/var/folders/abc/T/proj"));
        assert!(normalized.contains(r#"path = "<ROOT>/lib/scopelib.jar""#));
    }

    /// A windows-style path with backslashes is serialized as a literal
    /// (single-quoted) toml string and must be re-quoted after substitution.
    #[test]
    fn sanitize_toml_requotes_single_quoted_path() {
        let input = r#"[[package]]
path = 'C:\Users\foo\AppData\Local\Temp\proj\lib\scopelib.jar'
annotations = []
"#;
        let normalized =
            sanitize_toml(input, Path::new(r"C:\Users\foo\AppData\Local\Temp\proj"));
        assert!(normalized.contains(r#"path = "<ROOT>/lib/scopelib.jar""#));
    }

    /// The canonicalized (`/private/var/...`) form must also be substituted
    /// when it differs from the symlinked (`/var/...`) root.
    #[test]
    fn sanitize_toml_substitutes_canonical_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_s = root.to_string_lossy().replace('\\', "/");
        let canonical_s = std::fs::canonicalize(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");

        let mut input = format!("path = \"{canonical_s}/lib/x.jar\"\n");
        if canonical_s != root_s {
            input.push_str(&format!("path2 = \"{root_s}/lib/y.jar\"\n"));
        }

        let normalized = sanitize_toml(&input, root);
        assert!(
            normalized.contains(r#"path = "<ROOT>/lib/x.jar""#),
            "canonical path should be substituted, got:\n{normalized}"
        );
        assert!(normalized.contains("<ROOT>"));
    }

    /// Lines that already contain a double-quote (basic toml strings) must not
    /// be altered by the single-quote re-quoting pass.
    #[test]
    fn sanitize_toml_leaves_basic_strings_untouched() {
        let input = "path = \"<ROOT>/lib/x.jar\"";
        assert_eq!(sanitize_toml(input, Path::new("/var/folders/p")), input);
    }

    /// Content without the project root must pass through unchanged.
    #[test]
    fn sanitize_toml_passthrough_without_root() {
        let input = "scope = \"compile\"\nannotations = []";
        assert_eq!(sanitize_toml(input, Path::new("/var/folders/p")), input);
    }

    /// A fixture directory is copied into a fresh temp dir under `project/`.
    #[test]
    fn copy_fixture_copies_fixture_into_project_dir() {
        let src = tempfile::tempdir().unwrap();
        let content = "[project]\nname = \"x\"\n";
        std::fs::write(src.path().join("lazy-java.toml"), content).unwrap();
        std::fs::create_dir_all(src.path().join("src")).unwrap();
        std::fs::write(src.path().join("src").join("Main.java"), "class Main {}").unwrap();

        let tmp = copy_fixture(src.path()).unwrap();
        let dest = tmp.path().join("project");
        assert_eq!(
            std::fs::read_to_string(dest.join("lazy-java.toml")).unwrap(),
            content
        );
        assert!(dest.join("src").join("Main.java").exists());
    }
}