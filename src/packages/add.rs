use colored::Colorize;

use crate::{
    Context, args::AddArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn add(add_args: &AddArgs, ctx: &mut Context) -> Result<(), LazyJavaError> {
        if ctx.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        ctx.config.add_package(&add_args, ctx)?;

        ctx.config.write(&ctx.root)?;

        if !ctx.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
