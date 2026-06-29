use colored::Colorize;

use crate::{
    Context, LazyJava, args::RemoveArgs, config::Config, lazy_java_error::LazyJavaError,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(remove_args: &RemoveArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        if remove_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        let mut config = Config::fetch(&ctx.root)?;

        config.remove_package(remove_args, &ctx.root, &ctx.lib)?;

        config.write(&ctx.root)?;
        if !remove_args.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
