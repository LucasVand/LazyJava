use inquire::Select;

use crate::{find_main::find_main_classes, lazy_java::LazyJava, run::run_error::RunError};

impl LazyJava {
    pub fn interactive_find_main(&self) -> Result<String, RunError> {
        let options =
            find_main_classes(&self.root).map_err(|e| RunError::MainClassSearchError(e))?;

        let configured_options: Vec<String> = options
            .into_iter()
            .map(|op| {
                return op.full_package_name;
            })
            .collect();

        let res = Select::new("Select a Main Class to Run: ", configured_options)
            .without_help_message()
            .without_filtering()
            .prompt()
            .map_err(|_e| RunError::PromptError)?;

        return Ok(res);
    }
}
