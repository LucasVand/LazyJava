use colored::Colorize;

use crate::{
    Context, LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(remove_args: &RemoveArgs, ctx: Context) -> Result<(), LazyJavaError> {
        if ctx.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        let (inc, mut exc) = ctx.decompose();
        exc.config.remove_package(remove_args, &inc)?;

        let ctx = Context::compose(inc, exc);

        ctx.config.write(&ctx.root)?;
        if !ctx.dry_run {
            Classpath::generate(&ctx)?;
        }

        Ok(())
    }
}
