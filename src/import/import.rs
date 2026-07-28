use std::env;

use crate::{
    LazyJava,
    args::{ImportArgs, ImportCommand, LazyJavaArgs},
    import::{ImportError, pom::import_pom},
    lazy_java_error::LazyJavaError,
};

impl LazyJava {
    pub fn import(args: &ImportArgs, all_args: &LazyJavaArgs) -> Result<(), LazyJavaError> {
        let dry_run = all_args.global_args.dry_run;
        let res: Result<(), ImportError> = match &args.command {
            ImportCommand::Pom { args } => {
                let root = env::current_dir()?;

                import_pom(&root, args, dry_run)
            }
        };

        Ok(res?)
    }
}
