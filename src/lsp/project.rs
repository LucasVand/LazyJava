use std::{fs::write, io, path::Path};

use crate::config::Config;

pub struct DotProject {}

impl DotProject {
    pub fn generate(root: &Path, config: &Config) -> Result<(), io::Error> {
        let def = "Unamed-Project".to_string();
        let name: &str = config.project.name.as_ref().unwrap_or(&def);
        write(
            root.join(".project"),
            format!(
                r#" 
<?xml version="1.0" encoding="UTF-8"?>
<projectDescription>
	<name>{name}</name>
	<comment></comment>
	<projects>
	</projects>
	<natures>
		<nature>org.eclipse.jdt.core.javanature</nature>
	</natures>
	<filteredResources>
		<filter>
			<id>1779043104810</id>
			<name></name>
			<type>30</type>
			<matcher>
				<id>org.eclipse.core.resources.regexFilterMatcher</id>
				<arguments>node_modules|\.git|__CREATED_BY_JAVA_LANGUAGE_SERVER__</arguments>
			</matcher>
		</filter>
	</filteredResources>
</projectDescription>
"#
            ),
        )
    }
}
