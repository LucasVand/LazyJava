use crate::{
    Context, args::AddArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lsp::sync_lsp_config,
};

impl LazyJava {
    pub fn add(add_args: &AddArgs, ctx: Context) -> Result<(), LazyJavaError> {
        let (inc, mut exc) = ctx.decompose();

        exc.config.add_package(&add_args, &inc)?;
        let ctx = Context::compose(inc, exc);

        if !ctx.dry_run {
            sync_lsp_config(&ctx)?;
        }

        Ok(())
    }
}
