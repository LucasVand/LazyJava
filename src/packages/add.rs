use colored::Colorize;

use crate::{
    Context, args::AddArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn add(add_args: &AddArgs, ctx: &mut Context) -> Result<(), LazyJavaError> {
        if add_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        let config = &mut ctx.config;

        config.add_package(&add_args, &ctx.root, &ctx.lib)?;

        config.write(&ctx.root)?;

        if !add_args.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
