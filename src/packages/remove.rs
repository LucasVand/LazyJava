use colored::Colorize;

use crate::{
    LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(&self, remove_args: &RemoveArgs) -> Result<(), LazyJavaError> {
        if remove_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }
        self.assert_build_lib_src()?;

        let mut lockfile = LockFile::fetch(&self.root)?;

        println!(
            "{} {} {}",
            "Removing".green().bold(),
            remove_args.group,
            remove_args.artifact
        );
        let package = lockfile.remove_package(&remove_args.group, &remove_args.artifact)?;
        println!("    {} {} ", "Removed".green().bold(), package.id);

        if !remove_args.dry_run {
            lockfile.write(&self.root)?;
        }

        lockfile.validate_current_packages(&self.lib, remove_args.dry_run)?;

        if !remove_args.dry_run {
            Classpath::generate(self)?;
        }

        Ok(())
    }
}
