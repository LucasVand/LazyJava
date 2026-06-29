use crate::{
    Context,
    args::{LazyJavaArgs, LazyJavaCommand},
    lazy_java_error::LazyJavaError,
};
pub struct LazyJava;

impl LazyJava {
    pub fn execute(args: LazyJavaArgs) -> Result<(), LazyJavaError> {
        // commands that do not require contexts go here
        match &args.command {
            LazyJavaCommand::Create { args } => Self::create(args)?,
            _ => (),
        }

        let context = Context::new(&args)?;

        // commands that require contexts go here
        match &args.command {
            LazyJavaCommand::Run { args } => Self::run(args, &context)?,
            LazyJavaCommand::Build { args } => Self::build(args, &context)?,
            LazyJavaCommand::Clean {} => Self::clean(&context)?,
            LazyJavaCommand::Find { args } => Self::find(args, &context)?,
            LazyJavaCommand::Add { args } => Self::add(args, &context)?,
            LazyJavaCommand::Remove { args } => Self::remove(args, &context)?,
            LazyJavaCommand::Sync { args } => Self::sync(args, &context)?,
            _ => (),
        };
        Ok(())
    }
}
