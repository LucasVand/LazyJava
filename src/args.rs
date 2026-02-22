use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LazyJavaArgs {
    /// Operation to execute
    #[command(subcommand)]
    pub command: LazyJavaCommand,

    #[command(flatten)]
    pub global_args: LazyJavaGlobalArgs,
}

#[derive(Subcommand)]
pub enum LazyJavaCommand {
    /// Compile and run a java main class
    Run {
        #[command(flatten)]
        args: RunArgs,
    },
    /// Compile a java project
    Build {},
    /// Clean the java build folder
    Clean {},
}
#[derive(Debug, Parser)]
pub struct RunArgs {
    /// The main class to run
    pub class: String,

    /// Skip the compile step and run from build folder
    #[arg(long = "no-build", short = 'n')]
    pub no_build: bool,

    #[arg(long = "args", short = 'a', num_args = 1..)]
    pub args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct LazyJavaGlobalArgs {
    #[arg(short, long, global = true)]
    pub verbose: bool,
}
