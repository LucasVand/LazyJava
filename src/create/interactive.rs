use inquire::{Confirm, Text};

use crate::create::CreateError;

pub fn interactive_project_name() -> Result<String, CreateError> {
    log::debug!("Prompting user for project name");
    let name = Text::new("Project name:").prompt().map_err(|e| {
        log::error!("Failed to get project name from user: {}", e);
        CreateError::ProjectNameError
    })?;

    log::debug!("User entered project name: {}", name);
    Ok(name)
}

pub fn interactive_git_init_name() -> Result<bool, CreateError> {
    log::debug!("Prompting user for git initialization");
    let init = Confirm::new("Initialize git repository (y/n):")
        .prompt()
        .map_err(|e| {
            log::error!("Failed to get git initialization choice from user: {}", e);
            CreateError::ProjectNameError
        })?;

    log::debug!(
        "User chose to {} initialize git",
        if init { "" } else { "not " }
    );
    Ok(init)
}
