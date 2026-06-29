use colored::Colorize;

use crate::{
    Context, LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(remove_args: &RemoveArgs, ctx: &mut Context) -> Result<(), LazyJavaError> {
        if remove_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        ctx.config
            .remove_package(remove_args, &ctx.root, &ctx.lib)?;

        ctx.config.write(&ctx.root)?;
        if !remove_args.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
