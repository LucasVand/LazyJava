#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        config::{
            Config, ConfigDependancy, ConfigError, ConfigProject, ConfigResources, ConfigSetup,
        },
        maven_central::{MavenIdBuf, PartialMavenIdBuf},
    };

    fn make_deps(entries: &[(&str, &str, &str)]) -> HashMap<PartialMavenIdBuf, ConfigDependancy> {
        entries
            .iter()
            .map(|&(group, artifact, version)| {
                let id = MavenIdBuf::new(group, artifact, version);
                let key: PartialMavenIdBuf = id.clone().into();
                (key, ConfigDependancy { id })
            })
            .collect()
    }

    #[test]
    fn config_round_trip() {
        let config = Config {
            project: ConfigProject {
                name: "my-project".into(),
                group: Some("com.example".into()),
                artifact: Some("my-app".into()),
                version: Some("1.0.0".into()),
            },
            setup: ConfigSetup::default(),
            dependancies: make_deps(&[
                ("org.springframework", "spring-core", "6.0.0"),
                ("junit", "junit", "4.13.2"),
            ]),
            resources: ConfigResources {
                exclude: Vec::new(),
            },
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.project.name, "my-project");
        assert_eq!(deserialized.project.group.unwrap(), "com.example");
        assert_eq!(deserialized.project.artifact.unwrap(), "my-app");
        assert_eq!(deserialized.project.version.unwrap(), "1.0.0");
        assert_eq!(deserialized.dependancies.len(), 2);
        assert!(
            deserialized
                .dependancies
                .contains_key(&PartialMavenIdBuf::new(
                    "org.springframework",
                    "spring-core"
                ))
        );
        assert!(
            deserialized
                .dependancies
                .contains_key(&PartialMavenIdBuf::new("junit", "junit"))
        );
    }

    #[test]
    fn empty_dependencies_round_trip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert!(deserialized.dependancies.is_empty());
    }

    #[test]
    fn default_config_has_empty_dependencies() {
        let config = Config::default();
        assert!(config.dependancies.is_empty());
    }

    #[test]
    fn dependencies_serialize_as_map_keyed_by_artifact() {
        let config = Config {
            project: ConfigProject::default(),
            setup: ConfigSetup::default(),
            dependancies: make_deps(&[("com.example", "my-lib", "2.0.0")]),
            resources: ConfigResources::default(),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("my-lib"));
        assert!(toml_str.contains("com.example"));
        assert!(toml_str.contains("2.0.0"));
    }

    #[test]
    fn multiple_dependencies_round_trip() {
        let config = Config {
            project: ConfigProject::default(),
            setup: ConfigSetup::default(),
            dependancies: make_deps(&[
                ("group.a", "artifact-a", "1.0.0"),
                ("group.b", "artifact-b", "2.0.0"),
                ("group.c", "artifact-c", "3.0.0"),
            ]),
            resources: ConfigResources::default(),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.dependancies.len(), 3);
        assert!(
            deserialized
                .dependancies
                .contains_key(&PartialMavenIdBuf::new("group.a", "artifact-a"))
        );
        assert!(
            deserialized
                .dependancies
                .contains_key(&PartialMavenIdBuf::new("group.b", "artifact-b"))
        );
        assert!(
            deserialized
                .dependancies
                .contains_key(&PartialMavenIdBuf::new("group.c", "artifact-c"))
        );
    }

    #[test]
    fn deduplication_by_group_and_artifact() {
        let mut deps = make_deps(&[("g", "a", "1.0.0")]);

        let new_id = MavenIdBuf::new("g", "a", "2.0.0");
        deps.insert(new_id.clone().into(), new_id.clone().into());

        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps.get(&PartialMavenIdBuf::new("g", "a"))
                .unwrap()
                .id
                .version,
            "2.0.0"
        );
    }

    #[test]
    fn reject_empty_group() {
        let toml_str = r#"
[dependancies]
my-lib = { group = "", version = "1.0.0" }
"#;
        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_version() {
        let toml_str = r#"
[dependancies]
my-lib = { group = "com.example", version = "" }
"#;
        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn reject_empty_artifact_key() {
        let toml_str = r#"
[dependancies]
"" = { group = "com.example", version = "1.0.0" }
"#;
        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn write_and_fetch_round_trip() -> Result<(), ConfigError> {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let config = Config {
            resources: ConfigResources::default(),
            project: ConfigProject {
                name: "test-project".into(),
                ..Default::default()
            },
            setup: ConfigSetup::default(),
            dependancies: make_deps(&[("com.test", "test-lib", "0.1.0")]),
        };

        config.write(root)?;

        let loaded = Config::fetch(root)?;
        assert_eq!(loaded.project.name, "test-project");
        assert_eq!(loaded.dependancies.len(), 1);
        assert!(
            loaded
                .dependancies
                .contains_key(&PartialMavenIdBuf::new("com.test", "test-lib"))
        );

        Ok(())
    }

    #[test]
    fn write_empty_dependencies() -> Result<(), ConfigError> {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let config = Config::default();
        config.write(root)?;

        let loaded = Config::fetch(root)?;
        assert!(loaded.dependancies.is_empty());

        Ok(())
    }

    #[test]
    fn fetch_returns_default_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::fetch(dir.path()).unwrap();
        assert!(config.dependancies.is_empty());
    }
}
