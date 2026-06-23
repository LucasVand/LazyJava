use crate::{
    LazyJava, args::RemoveArgs, lazy_java_error::LazyJavaError, lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn remove(&self, remove_args: &RemoveArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let mut lockfile = LockFile::fetch(&self.root)?;

        let before = lockfile.packages.len();
        lockfile.remove_package(
            &remove_args.group,
            &remove_args.artifact,
            remove_args.remove_transitive,
        )?;
        let removed = before - lockfile.packages.len();

        lockfile.write(&self.root)?;

        lockfile.validate_current_packages(&self.lib)?;

        Classpath::generate(self)?;

        if removed > 1 {
            println!(
                "Removed {}:{} (+ {} transitive {})",
                remove_args.group,
                remove_args.artifact,
                removed - 1,
                if removed - 1 == 1 { "dependency" } else { "dependencies" },
            );
        } else {
            println!("Removed {}:{}", remove_args.group, remove_args.artifact);
        }

        Ok(())
    }
}
