use std::{
    io,
    path::Path,
};

use crate::utils::fs;

pub struct DotSettings {}

impl DotSettings {
    pub fn generate(root: &Path) -> Result<(), io::Error> {
        let dot_settings = root.join(".settings");
        if !fs::exists(&dot_settings)? {
            fs::create_dir(&dot_settings)?;
        }
        fs::write(
            root.join(".settings/org.eclipse.core.prefs"),
            r#"eclipse.preferences.version=1
org.eclipse.jdt.core.compiler.source=25
org.eclipse.jdt.core.compiler.compliance=25
org.eclipse.jdt.core.compiler.codegen.targetPlatform=25
"#,
        )
    }
}
