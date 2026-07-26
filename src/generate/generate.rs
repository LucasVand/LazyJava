use crate::{
    Context, LazyJava,
    args::{GenerateArgs, GenerateCommand},
    generate::pom::generate_pom,
    lazy_java_error::LazyJavaError,
};

impl LazyJava {
    pub fn generate(args: &GenerateArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        match args.command {
            GenerateCommand::Pom => generate_pom(ctx)?,
        }

        Ok(())
    }
}
