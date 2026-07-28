use crate::{
    Context, args::SyncArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lock_file::LockFile, lsp::sync_lsp_config,
};

impl LazyJava {
    pub fn sync(_sync_args: &SyncArgs, ctx: Context) -> Result<(), LazyJavaError> {
        let (inc, exc) = ctx.decompose();

        let mut lockfile = LockFile::fetch(&inc.root)?;

        exc.config.sync_lock_file(&mut lockfile, &inc)?;

        let ctx = Context::compose(inc, exc);

        if !ctx.dry_run {
            sync_lsp_config(&ctx)?;
        }

        Ok(())
    }
}
