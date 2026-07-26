use std::env;

use crate::{
    LazyJava,
    args::{ImportArgs, ImportCommand},
    import::{ImportError, pom::import_pom},
    lazy_java_error::LazyJavaError,
};

impl LazyJava {
    pub fn import(args: &ImportArgs) -> Result<(), LazyJavaError> {
        let res: Result<(), ImportError> = match &args.command {
            ImportCommand::Pom { args } => {
                let root = env::current_dir()?;

                import_pom(&root, args)
            }
        };

        Ok(res?)
    }
}
