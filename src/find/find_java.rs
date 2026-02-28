use std::io;

use crate::{args::FindArgs, find_main::find_main_classes, lazy_java::LazyJava};

impl LazyJava {
    pub fn find(&self, _args: &FindArgs) -> Result<(), io::Error> {
        let mains = find_main_classes(&self.src)?;

        for main in mains {
            println!(
                "- {}, Package: {}, File: {}",
                main.classname,
                main.full_package_name,
                main.path.to_str().unwrap()
            );
        }

        return Ok(());
    }
}
