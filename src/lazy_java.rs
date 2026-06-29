use crate::{
    Context,
    args::{LazyJavaArgs, LazyJavaCommand},
    lazy_java_error::LazyJavaError,
};
pub struct LazyJava;

impl LazyJava {
    pub fn execute(args: LazyJavaArgs) -> Result<(), LazyJavaError> {
        // commands that do not require contexts go here
        if let LazyJavaCommand::Create { args } = &args.command {
            Self::create(args)?;
        }

        let mut context = Context::new(&args)?;

        // commands that require contexts go here
        match &args.command {
            LazyJavaCommand::Run { args } => Self::run(args, &context)?,
            LazyJavaCommand::Build { args } => Self::build(args, &context)?,
            LazyJavaCommand::Clean {} => Self::clean(&context)?,
            LazyJavaCommand::Find { args } => Self::find(args, &context)?,
            LazyJavaCommand::Add { args } => Self::add(args, &mut context)?,
            LazyJavaCommand::Remove { args } => Self::remove(args, &mut context)?,
            LazyJavaCommand::Sync { args } => Self::sync(args, &mut context)?,
            LazyJavaCommand::Create { args: _ } => panic!("Should be handled earlier"),
        };
        Ok(())
    }
}
