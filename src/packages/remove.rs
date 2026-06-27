use colored::Colorize;

use crate::{
    LazyJava, args::RemoveArgs, config::Config, lazy_java_error::LazyJavaError,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(&self, remove_args: &RemoveArgs) -> Result<(), LazyJavaError> {
        if remove_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }
        self.assert_build_lib_src()?;

        let mut config = Config::fetch(&self.root)?;

        config.remove_package(remove_args, &self.root, &self.lib)?;

        config.write(&self.root)?;
        if !remove_args.dry_run {
            Classpath::generate(self)?;
        }

        Ok(())
    }
}
