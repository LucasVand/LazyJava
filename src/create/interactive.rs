use inquire::{Confirm, Text};

use crate::{create::create_project::CreateProjectError, lazy_java_error::LazyJavaError};

pub fn interactive_project_name() -> Result<String, LazyJavaError> {
    let name = Text::new("Project name:")
        .prompt()
        .map_err(|_e| CreateProjectError::ProjectNameError)?;

    return Ok(name);
}

pub fn interactive_git_init_name() -> Result<bool, LazyJavaError> {
    let init = Confirm::new("Initalize git repository (y/n):")
        .prompt()
        .map_err(|_e| CreateProjectError::ProjectNameError)?;

    return Ok(init);
}
