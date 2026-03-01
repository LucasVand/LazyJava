use inquire::Select;

use crate::{
    lazy_java::LazyJava, lazy_java_error::LazyJavaError, utils::find_main::find_main_classes,
};

impl LazyJava {
    pub fn interactive_find_main(&self) -> Result<String, LazyJavaError> {
        let options =
            find_main_classes(&self.root).map_err(|e| return LazyJavaError::CouldntFindMains(e))?;

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
            .map_err(|_e| LazyJavaError::PromptError)?;

        return Ok(res);
    }
}
