use crate::{
    Context, LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lsp::sync_lsp_config,
};

impl LazyJava {
    pub fn remove(remove_args: &RemoveArgs, ctx: Context) -> Result<(), LazyJavaError> {
        let (inc, mut exc) = ctx.decompose();
        exc.config.remove_package(remove_args, &inc)?;

        let ctx = Context::compose(inc, exc);

        if !ctx.dry_run {
            sync_lsp_config(&ctx)?;
        }

        Ok(())
    }
}
