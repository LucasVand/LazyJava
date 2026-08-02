use inquire::Select;

use crate::{
    Context,
    run::RunError,
    utils::{IOError, find_main::find_main_classes},
};

pub fn interactive_find_main(ctx: &Context) -> Result<String, RunError> {
    log::debug!("Finding main classes interactively");
    let exclude = if let Some(s) = ctx.config.setup()
        && let Some(list) = s.exclude()
    {
        list
    } else {
        Vec::new()
    };
    let options = find_main_classes(&ctx.src, &exclude)
        .map_err(|e| IOError::new("finding main classes", &ctx.src, e))?;
    log::debug!("Found {} main classes", options.len());

    if options.is_empty() {
        log::error!("No main classes found");
        return Err(RunError::NoMainClasses);
    }

    if options.len() == 1 {
        log::debug!(
            "Only one main class found: {}",
            options[0].full_package_name
        );
        return Ok(options[0].full_package_name.clone());
    }

    let configured_options: Vec<String> =
        options.into_iter().map(|op| op.full_package_name).collect();

    let res = Select::new("Select a Main Class to Run: ", configured_options)
        .without_help_message()
        .without_filtering()
        .prompt()
        .map_err(|_e| RunError::PromptError)?;
    log::debug!("User selected main class: {}", res);

    Ok(res)
}
