use colored::Colorize;

use crate::{
    lazy_java::LazyJava, lazy_java_error::LazyJavaError, lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn sync(&self) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let lockfile = LockFile::fetch(&self.root)?;

        println!("{} dependancies", "Syncing".green().bold());
        lockfile.validate_current_packages(&self.lib)?;

        println!("    {} dependancies", "Synced".green().bold());

        Classpath::generate(self)?;

        Ok(())
    }
}
