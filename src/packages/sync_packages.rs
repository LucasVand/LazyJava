use colored::Colorize;

use crate::{
    Context, args::SyncArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lock_file::LockFile, lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn sync(sync_args: &SyncArgs, ctx: &mut Context) -> Result<(), LazyJavaError> {
        if sync_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        let config = &mut ctx.config;
        let mut lockfile = LockFile::fetch(&ctx.root)?;

        config.sync_lock_file(&mut lockfile, &ctx.root, &ctx.lib, sync_args.dry_run)?;

        config.write(&ctx.root)?;

        if !sync_args.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
