use colored::Colorize;

use crate::{
    Context, args::AddArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lsp::sync_lsp_config,
};

impl LazyJava {
    pub fn add(add_args: &AddArgs, ctx: Context) -> Result<(), LazyJavaError> {
        if ctx.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }
        let (inc, mut exc) = ctx.decompose();

        exc.config.add_package(&add_args, &inc)?;
        let ctx = Context::compose(inc, exc);

        ctx.config.write(&ctx.root)?;

        if !ctx.dry_run {
            sync_lsp_config(&ctx)?;
        }

        Ok(())
    }
}
