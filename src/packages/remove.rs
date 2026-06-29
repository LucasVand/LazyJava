use colored::Colorize;

use crate::{
    Context, LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(remove_args: &RemoveArgs, ctx: &mut Context) -> Result<(), LazyJavaError> {
        if ctx.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        ctx.config.remove_package(remove_args, ctx)?;

        ctx.config.write(&ctx.root)?;
        if !ctx.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
