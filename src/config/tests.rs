#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::{ConfigError, ConfigTomlEdit};

    fn config_toml() -> &'static str {
        r#"[project]
name = "my-project"
group = "com.example"
artifact = "my-app"
version = "1.0.0"

[dependencies]
spring-core = { group_id = "org.springframework", version = "6.0.0" }
junit = { group_id = "junit", version = "4.13.2" }
"#
    }

    #[test]
    fn config_round_trip() {
        let toml_str = config_toml();
        let loaded = ConfigTomlEdit::parse(toml_str).unwrap();

        assert_eq!(loaded.project().unwrap().name().unwrap(), "my-project");
        assert_eq!(loaded.project().unwrap().group().unwrap(), "com.example");
        assert_eq!(loaded.project().unwrap().artifact().unwrap(), "my-app");
        assert_eq!(loaded.project().unwrap().version().unwrap(), "1.0.0");
        assert!(loaded.dependencies().is_some());
        assert!(loaded.dependencies().unwrap().contains_key("spring-core"));
        assert!(loaded.dependencies().unwrap().contains_key("junit"));
    }

    #[test]
    fn empty_dependencies_round_trip() {
        let toml_str = r#"[project]
name = "empty"
"#;
        let loaded = ConfigTomlEdit::parse(toml_str).unwrap();
        assert!(loaded.dependencies().is_none_or(|d| d.is_empty()));
    }

    #[test]
    fn default_config_has_empty_dependencies() {
        let loaded = ConfigTomlEdit::parse("").unwrap();
        assert!(loaded.dependencies().is_none_or(|d| d.is_empty()));
    }

    #[test]
    fn dependencies_serialize_as_map_keyed_by_artifact() {
        let toml_str = r#"[project]
name = "test"

[dependencies]
my-lib = { group = "com.example", version = "2.0.0" }
"#;
        let loaded = ConfigTomlEdit::parse(toml_str).unwrap();
        let deps = loaded.dependencies().unwrap();
        let entry = deps.get("my-lib").unwrap();
        assert_eq!(entry.group().unwrap(), "com.example");
        assert_eq!(entry.version().unwrap(), "2.0.0");
    }

    #[test]
    fn write_and_fetch_round_trip() -> Result<(), ConfigError> {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let toml_str = r#"[project]
name = "test-project"

[dependencies]
test-lib = { group = "com.test", version = "0.1.0" }
"#;
        let config = ConfigTomlEdit::parse(toml_str)?;
        config.write(root)?;

        let loaded = ConfigTomlEdit::fetch(root)?;
        assert_eq!(loaded.project().unwrap().name().unwrap(), "test-project");
        assert!(!loaded.dependencies().unwrap().is_empty());

        Ok(())
    }

    #[test]
    fn write_empty_dependencies() -> Result<(), ConfigError> {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let config = ConfigTomlEdit::parse(
            r#"[project]
name = "empty"
"#,
        )?;
        config.write(root)?;

        let loaded = ConfigTomlEdit::fetch(root)?;
        assert!(loaded.dependencies().is_none_or(|d| d.is_empty()));

        Ok(())
    }

    #[test]
    fn fetch_errors_when_no_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = ConfigTomlEdit::fetch(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn comments_preserved_round_trip() {
        let toml_str = r#"# Top-level comment
[project] # project comment
name = "my-project" # name comment
group = "com.example"
# group comment
artifact = "my-app"
version = "1.0.0"

# Dependencies section comment
[dependencies]
# spring-core comment
spring-core = { group = "org.springframework", version = "6.0.0" }
junit = { group = "junit", version = "4.13.2" } # junit inline
"#;
        let loaded = ConfigTomlEdit::parse(toml_str).unwrap();
        let output = loaded.to_toml_string();

        assert!(output.contains("# Top-level comment"), "top comment");
        assert!(output.contains("# name comment"), "inline name comment");
        assert!(
            output.contains("# group comment"),
            "standalone group comment"
        );
        assert!(
            output.contains("# Dependencies section comment"),
            "section comment"
        );
        assert!(output.contains("# spring-core comment"), "dep comment");
        assert!(output.contains("# junit inline"), "dep inline comment");
        assert!(
            output.contains("# project comment"),
            "section inline comment"
        );
    }

    #[test]
    fn ordering_preserved_round_trip() {
        let toml_str = r#"[project]
name = "ordered"

[dependencies]
zeta = { group = "z.org", version = "1.0.0" }
alpha = { group = "a.org", version = "2.0.0" }
delta = { group = "d.org", version = "3.0.0" }
"#;
        let loaded = ConfigTomlEdit::parse(toml_str).unwrap();
        let output = loaded.to_toml_string();

        let zeta_pos = output.find("zeta").unwrap();
        let alpha_pos = output.find("alpha").unwrap();
        let delta_pos = output.find("delta").unwrap();

        assert!(zeta_pos < alpha_pos, "zeta before alpha");
        assert!(alpha_pos < delta_pos, "alpha before delta");
    }

    #[test]
    fn comments_preserved_after_add() {
        let toml_str = r#"[project]
name = "test"

# Existing deps
[dependencies]
existing = { group = "com.existing", version = "1.0.0" }
"#;
        let mut config = ConfigTomlEdit::parse(toml_str).unwrap();
        let mut deps = config.dependencies_mut().get_or_insert(HashMap::new());
        let mut value = deps.insert_empty("new-dep");
        value.group_mut().replace("com.new".to_string());
        value.version_mut().replace("2.0.0".to_string());
        let output = config.to_toml_string();

        assert!(
            output.contains("# Existing deps"),
            "existing comment preserved"
        );
        assert!(
            output.contains("existing = { group = \"com.existing\", version = \"1.0.0\" }"),
            "existing dep preserved"
        );
        assert!(
            output.contains("new-dep = { group = \"com.new\", version = \"2.0.0\" }"),
            "new dep present"
        );
    }

    #[test]
    fn ordering_preserved_after_add() {
        let toml_str = r#"[project]
name = "test"

[dependencies]
oldest = { group = "o.org", version = "1.0.0" }
older = { group = "r.org", version = "2.0.0" }
"#;
        let mut config = ConfigTomlEdit::parse(toml_str).unwrap();
        let mut deps = config.dependencies_mut().get_or_insert(HashMap::new());
        let mut value = deps.insert_empty("newest");
        value.group_mut().replace("n.org".to_string());
        value.version_mut().replace("3.0.0".to_string());
        let output = config.to_toml_string();

        let oldest_pos = output.find("oldest").unwrap();
        let older_pos = output.find("older").unwrap();
        let newest_pos = output.find("newest").unwrap();

        assert!(oldest_pos < older_pos, "oldest before older");
        assert!(
            older_pos < newest_pos,
            "older before newest (inserted at end)"
        );
    }

    #[test]
    fn comments_and_ordering_after_remove() {
        let toml_str = r#"# Header
[project]
name = "test"

# Deps header
[dependencies]
aaa = { group = "a.org", version = "1.0.0" } # a comment
# b comment
bbb = { group = "b.org", version = "2.0.0" }
ccc = { group = "c.org", version = "3.0.0" }
"#;
        let mut config = ConfigTomlEdit::parse(toml_str).unwrap();
        if let Some(mut deps) = config.dependencies_mut().get_mut() {
            deps.remove("bbb");
        }
        let output = config.to_toml_string();

        assert!(output.contains("# Header"), "header comment preserved");
        assert!(
            output.contains("# Deps header"),
            "section comment preserved"
        );
        assert!(
            output.contains("# a comment"),
            "inline comment on aaa preserved"
        );
        assert!(!output.contains("bbb"), "bbb removed");
        let aaa_pos = output.find("aaa").unwrap();
        let ccc_pos = output.find("ccc").unwrap();
        assert!(aaa_pos < ccc_pos, "aaa before ccc after remove");
    }
}
