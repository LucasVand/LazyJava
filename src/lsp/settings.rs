use std::io;

use crate::{Context, utils::fs, utils::jdk_version::desired_jdk_version};

pub struct DotSettings {}

impl DotSettings {
    pub fn generate(ctx: &Context) -> Result<(), io::Error> {
        let root = &ctx.root;
        let version = desired_jdk_version(None, Some(ctx));
        let dot_settings = root.join(".settings");
        if !fs::exists(&dot_settings)? {
            fs::create_dir(&dot_settings)?;
        }
        fs::write(
            root.join(".settings/org.eclipse.core.prefs"),
            format!(
                "eclipse.preferences.version=1\n\
                 org.eclipse.jdt.core.compiler.source={version}\n\
                 org.eclipse.jdt.core.compiler.compliance={version}\n\
                 org.eclipse.jdt.core.compiler.codegen.targetPlatform={version}\n"
            ),
        )
    }
}
