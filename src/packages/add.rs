use colored::Colorize;

use crate::{
    args::AddArgs, config::Config, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn add(&self, add_args: &AddArgs) -> Result<(), LazyJavaError> {
        if add_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }
        self.assert_build_lib_src()?;

        let mut config = Config::fetch(&self.root)?;

        config.add_package(&add_args, &self.root, &self.lib)?;

        config.write(&self.root)?;
        if !add_args.dry_run {
            Classpath::generate(self)?;
        }

        Ok(())
    }
}
