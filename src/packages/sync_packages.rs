use colored::Colorize;

use crate::{
    Context, args::SyncArgs, config::Config, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    lock_file::LockFile, lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn sync(sync_args: &SyncArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        if sync_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        let config = Config::fetch(&ctx.root)?;
        let mut lockfile = LockFile::fetch(&ctx.root)?;

        config.sync_lock_file(&mut lockfile, &ctx.root, &ctx.lib, sync_args.dry_run)?;

        config.write(&ctx.root)?;

        if !sync_args.dry_run {
            Classpath::generate(ctx)?;
        }

        Ok(())
    }
}
