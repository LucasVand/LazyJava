//! Shared helpers for the integration test suite.

use std::path::Path;

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
}