use inquire::{Confirm, Text};

use crate::{create::create_project::CreateProjectError, lazy_java_error::LazyJavaError};

pub fn interactive_project_name() -> Result<String, LazyJavaError> {
    log::debug!("Prompting user for project name");
    let name = Text::new("Project name:").prompt().map_err(|e| {
        log::error!("Failed to get project name from user: {}", e);
        CreateProjectError::ProjectNameError
    })?;

    log::debug!("User entered project name: {}", name);
    return Ok(name);
}

pub fn interactive_git_init_name() -> Result<bool, LazyJavaError> {
    log::debug!("Prompting user for git initialization");
    let init = Confirm::new("Initalize git repository (y/n):")
        .prompt()
        .map_err(|e| {
            log::error!("Failed to get git initialization choice from user: {}", e);
            CreateProjectError::ProjectNameError
        })?;

    log::debug!(
        "User chose to {} initialize git",
        if init { "" } else { "not " }
    );
    return Ok(init);
}
