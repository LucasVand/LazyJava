use colored::Colorize;

use crate::{
    LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(&self, remove_args: &RemoveArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let mut lockfile = LockFile::fetch(&self.root)?;

        println!(
            "{} {} {}",
            "Removing".green().bold(),
            remove_args.group,
            remove_args.artifact
        );
        let package = lockfile.remove_package(
            &remove_args.group,
            &remove_args.artifact,
            remove_args.remove_transitive,
        )?;
        println!("    {} {} ", "Removed".green().bold(), package.id);

        lockfile.write(&self.root)?;

        lockfile.validate_current_packages(&self.lib)?;

        Classpath::generate(self)?;

        Ok(())
    }
}
