use inquire::Select;

use crate::{
    lazy_java::LazyJava, lazy_java_error::LazyJavaError, utils::find_main::find_main_classes,
};

impl LazyJava {
    pub fn interactive_find_main(&self) -> Result<String, LazyJavaError> {
        log::debug!("Finding main classes interactively");
        let options =
            find_main_classes(&self.src).map_err(LazyJavaError::CouldntFindMains)?;
        log::debug!("Found {} main classes", options.len());

        if options.is_empty() {
            log::error!("No main classes found");
            return Err(LazyJavaError::NoMainClasses);
        }

        if options.len() == 1 {
            log::debug!(
                "Only one main class found: {}",
                options[0].full_package_name
            );
            return Ok(options[0].full_package_name.clone());
        }

        let configured_options: Vec<String> = options
            .into_iter()
            .map(|op| {
                op.full_package_name
            })
            .collect();

        let res = Select::new("Select a Main Class to Run: ", configured_options)
            .without_help_message()
            .without_filtering()
            .prompt()
            .map_err(|_e| LazyJavaError::PromptError)?;
        log::debug!("User selected main class: {}", res);

        Ok(res)
    }
}
