use colored::Colorize;

use crate::{
    Context,
    args::SyncArgs,
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lock_file::LockFile,
    lsp::{DotProject, classpath::Classpath},
};

impl LazyJava {
    pub fn sync(_sync_args: &SyncArgs, ctx: Context) -> Result<(), LazyJavaError> {
        if ctx.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        let (inc, exc) = ctx.decompose();

        let mut lockfile = LockFile::fetch(&inc.root)?;

        exc.config.sync_lock_file(&mut lockfile, &inc)?;

        exc.config.write(&inc.root)?;

        let ctx = Context::compose(inc, exc);

        if !ctx.dry_run {
            Classpath::generate(&ctx)?;
            DotProject::generate(&ctx.root, &ctx.config)?;
        }

        Ok(())
    }
}
