use crate::{
    lazy_java::LazyJava, lazy_java_error::LazyJavaError, lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn sync(&self) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let lockfile = LockFile::fetch(&self.root)?;

        lockfile.validate_current_packages(&self.lib)?;

        Classpath::generate(self)?;

        Ok(())
    }
}
