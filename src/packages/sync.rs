use colored::Colorize;

use crate::{
    args::SyncArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError, lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn sync(&self, sync_args: &SyncArgs) -> Result<(), LazyJavaError> {
        if sync_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }

        self.assert_build_lib_src()?;

        let lockfile = LockFile::fetch(&self.root)?;

        lockfile.validate_current_packages(&self.lib, sync_args.dry_run)?;

        if !sync_args.dry_run {
            Classpath::generate(self)?;
        }

        Ok(())
    }
}
