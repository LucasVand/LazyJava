use std::fs;

use crate::{
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lock_file::LockFile,
    lsp::classpath::Classpath,
};

impl LazyJava {
    pub fn sync(&self) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let lockfile = LockFile::fetch(&self.root)?;

        let jars_before: Vec<_> = fs::read_dir(&self.lib)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "jar"))
            .collect();
        let count_before = jars_before.len();

        lockfile.validate_current_packages(&self.lib)?;

        let count_after = fs::read_dir(&self.lib)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "jar"))
            .count();

        let removed = count_before.saturating_sub(count_after);
        let added = count_after.saturating_sub(count_before);

        if removed > 0 {
            println!("Removed {} unused dependanc{}", removed, if removed == 1 { "y" } else { "ies" });
        }
        if added > 0 {
            println!("Added {} missing dependanc{}", added, if added == 1 { "y" } else { "ies" });
        }
        if removed == 0 && added == 0 {
            println!("Lock file is already in sync");
        }

        Classpath::generate(self)?;

        Ok(())
    }
}
