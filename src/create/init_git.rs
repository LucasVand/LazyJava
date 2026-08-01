use std::{
    env, fs, io,
    path::Path,
    process::{Command, ExitStatus},
};

fn git_command() -> Result<ExitStatus, io::Error> {
    log::debug!("Executing git init command");
    Command::new("git").args(["init"]).status()
}

pub fn git_init(project_path: &Path) -> Result<ExitStatus, io::Error> {
    log::debug!("Initializing git repository at {:?}", project_path);
    let current_path = env::current_dir()?;
    env::set_current_dir(project_path)?;

    let output = git_command()?;
    env::set_current_dir(&current_path)?;

    let gitignore = project_path.join(".gitignore");
    if gitignore.exists() {
        log::info!(".gitignore already exists, skipping");
    } else {
        fs::write(&gitignore, GITIGNORE_CONTENTS)?;
        log::debug!("Created .gitignore");
    }

    if output.success() {
        log::debug!("Git initialization successful");
    } else {
        log::warn!("Git initialization failed with status: {:?}", output.code());
    }

    Ok(output)
}
const GITIGNORE_CONTENTS: &str = r#"target/
*.class
*.jar
*.log
.env
.idea/
.vscode/
*.iml
.DS_Store
"#;
