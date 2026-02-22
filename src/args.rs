use clap::{Parser, Subcommand};

use crate::{BUILD_FOLDER, SRC_FOLDER};

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
    Build {
        #[command(flatten)]
        args: BuildArgs,
    },
    /// Clean the java build folder
    Clean {},
}
#[derive(Debug, Parser, Clone)]
pub struct RunArgs {
    /// The main class to run
    pub class: Option<String>,

    /// Skip the compile step and run from build folder
    #[arg(long = "no-build", short = 'n')]
    pub no_build: bool,

    #[arg(long = "args", short = 'a', num_args = 1..)]
    pub args: Vec<String>,

    #[command(flatten)]
    pub build_args: BuildArgs,
}

#[derive(Debug, Parser, Clone)]
pub struct BuildArgs {
    /// Where to find the java files to compile
    #[arg(long = "source", short = 's', default_value_t = SRC_FOLDER.to_string())]
    pub source: String,

    /// Where to save the compiled java files
    #[arg(long = "build", short = 'b', default_value_t = BUILD_FOLDER.to_string())]
    pub build: String,
}

#[derive(Debug, Parser)]
pub struct LazyJavaGlobalArgs {
    #[arg(short, long, global = true)]
    pub verbose: bool,
}
