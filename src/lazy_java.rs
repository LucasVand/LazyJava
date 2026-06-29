use crate::{
    Context,
    args::{
        AddArgs, BuildCommand, FindArgs, LazyJavaArgs, LazyJavaCommand, RemoveArgs, RunArgs,
        SyncArgs,
    },
    lazy_java_error::LazyJavaError,
};
pub struct LazyJava;

impl LazyJava {
    pub fn execute(cli_args: LazyJavaArgs) -> Result<(), LazyJavaError> {
        match &cli_args.command {
            LazyJavaCommand::Create { args } => Self::create(args),
            LazyJavaCommand::Build { args } => Self::build_internal(&cli_args, args),
            LazyJavaCommand::Clean {} => Self::clean_internal(&cli_args),
            LazyJavaCommand::Find { args } => Self::find_internal(&cli_args, args),
            LazyJavaCommand::Run { args } => Self::run_internal(&cli_args, args),
            LazyJavaCommand::Add { args } => Self::add_internal(&cli_args, args),
            LazyJavaCommand::Remove { args } => Self::remove_internal(&cli_args, args),
            LazyJavaCommand::Sync { args } => Self::sync_internal(&cli_args, args),
        }
    }

    fn build_internal(all_args: &LazyJavaArgs, args: &BuildCommand) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_build_lib_src()?;
        Self::build(args, &ctx)
    }

    fn clean_internal(all_args: &LazyJavaArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        Self::clean(&ctx)
    }

    fn find_internal(all_args: &LazyJavaArgs, args: &FindArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_src_exists()?;
        Self::find(args, &ctx)
    }

    fn run_internal(all_args: &LazyJavaArgs, args: &RunArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_build_lib_src()?;
        Self::run(args, &ctx)
    }

    fn add_internal(all_args: &LazyJavaArgs, args: &AddArgs) -> Result<(), LazyJavaError> {
        let mut ctx = Context::new(all_args)?;
        ctx.assert_build_lib_src()?;
        Self::add(args, &mut ctx)
    }

    fn remove_internal(all_args: &LazyJavaArgs, args: &RemoveArgs) -> Result<(), LazyJavaError> {
        let mut ctx = Context::new(all_args)?;
        ctx.assert_build_lib_src()?;
        Self::remove(args, &mut ctx)
    }

    fn sync_internal(all_args: &LazyJavaArgs, args: &SyncArgs) -> Result<(), LazyJavaError> {
        let mut ctx = Context::new(all_args)?;
        ctx.assert_build_lib_src()?;
        Self::sync(args, &mut ctx)
    }
}
