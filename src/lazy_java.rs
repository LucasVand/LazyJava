use colored::Colorize;

use crate::{
    Context,
    args::{
        AddArgs, BuildCommand, FindArgs, GenerateArgs, LazyJavaArgs, LazyJavaCommand, RemoveArgs,
        RunArgs, SyncArgs,
    },
    config::ConfigTomlEdit,
    lazy_java_error::LazyJavaError,
    utils::GlobalContext,
};
pub struct LazyJava;

impl LazyJava {
    pub fn execute(cli_args: LazyJavaArgs) -> Result<(), LazyJavaError> {
        GlobalContext::init(cli_args.global_args.dry_run);
        if cli_args.global_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }
        match &cli_args.command {
            LazyJavaCommand::Create { args } => Ok(Self::create(args, &cli_args)?),
            LazyJavaCommand::Build { args } => Self::build_internal(&cli_args, args),
            LazyJavaCommand::Clean {} => Self::clean_internal(&cli_args),
            LazyJavaCommand::Find { args } => Self::find_internal(&cli_args, args),
            LazyJavaCommand::Run { args } => Self::run_internal(&cli_args, args),
            LazyJavaCommand::Add { args } => Self::add_internal(&cli_args, args),
            LazyJavaCommand::Remove { args } => Self::remove_internal(&cli_args, args),
            LazyJavaCommand::Sync { args } => Self::sync_internal(&cli_args, args),
            LazyJavaCommand::Generate { args } => Self::generate_internal(&cli_args, args),
            LazyJavaCommand::Import { args } => Self::import(args),
        }
    }
    fn generate_internal(
        all_args: &LazyJavaArgs,
        args: &GenerateArgs,
    ) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_all()?;
        let ctx = ctx.assert_packages()?;
        Self::generate(args, &ctx)
    }

    fn build_internal(all_args: &LazyJavaArgs, args: &BuildCommand) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_all()?;
        let ctx = ctx.assert_packages()?;
        Self::build(args, &ctx)?;
        Ok(())
    }

    fn clean_internal(all_args: &LazyJavaArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ConfigTomlEdit::assert_config_file_exists(&ctx.root)?;
        Self::clean(&ctx)
    }

    fn find_internal(all_args: &LazyJavaArgs, args: &FindArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_src_exists()?;
        Self::find(args, &ctx)
    }

    fn run_internal(all_args: &LazyJavaArgs, args: &RunArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_all()?;
        let ctx = ctx.assert_packages()?;
        Self::run(args, &ctx)
    }

    fn add_internal(all_args: &LazyJavaArgs, args: &AddArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_all()?;
        Self::add(args, ctx)
    }

    fn remove_internal(all_args: &LazyJavaArgs, args: &RemoveArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_all()?;
        Self::remove(args, ctx)
    }

    fn sync_internal(all_args: &LazyJavaArgs, args: &SyncArgs) -> Result<(), LazyJavaError> {
        let ctx = Context::new(all_args)?;
        ctx.assert_all()?;
        Self::sync(args, ctx)
    }
}
