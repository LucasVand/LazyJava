use std::{
    env, io,
    path::Path,
    process::{Command, ExitStatus},
};

fn git_command() -> Result<ExitStatus, io::Error> {
    log::debug!("Executing git init command");
    if cfg!(target_os = "windows") {
        return Command::new("powershell")
            .args(["-Command", "git init"])
            .status();
    } else {
        return Command::new("sh").args(["-c", "git init"]).status();
    }
}

pub fn git_init(project_path: &Path) -> Result<ExitStatus, io::Error> {
    log::debug!("Initializing git repository at {:?}", project_path);
    let current_path = env::current_dir()?;
    env::set_current_dir(project_path)?;

    let output = git_command()?;
    env::set_current_dir(&current_path)?;

    if output.success() {
        log::debug!("Git initialization successful");
    } else {
        log::warn!("Git initialization failed with status: {:?}", output.code());
    }

    return Ok(output);
}
