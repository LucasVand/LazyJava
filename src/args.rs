use clap::{Parser, Subcommand};

use crate::{BUILD_FOLDER, LIB_FOLDER, SRC_FOLDER};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[derive(Debug, Clone)]
pub struct LazyJavaArgs {
    /// Operation to execute
    #[command(subcommand)]
    pub command: LazyJavaCommand,

    #[command(flatten)]
    pub global_args: LazyJavaGlobalArgs,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LazyJavaCommand {
    /// Compile and run a java main class
    Run {
        #[command(flatten)]
        args: RunArgs,
    },
    /// Compile a java project
    Build {
        #[command(flatten)]
        args: BuildCommand,
    },
    /// Clean the java build folder
    Clean {},
    /// Finds all main classes and prints them
    Find {
        #[command(flatten)]
        args: FindArgs,
    },
    Create {
        #[command(flatten)]
        args: CreateArgs,
    },
}
#[derive(Debug, Parser, Clone)]
pub struct RunArgs {
    /// The main class to run
    pub class: Option<String>,

    /// Skip the compile step and run from build folder
    #[arg(long = "no-build", short = 'n')]
    pub no_build: bool,

    #[arg(long = "args", short = 'a', num_args = 1.., allow_hyphen_values = true)]
    pub args: Vec<String>,

    #[command(flatten)]
    pub build_args: BuildArgs,
}

#[derive(Debug, Parser, Clone)]
pub struct BuildArgs {
    /// Rebuild all files
    #[arg(long = "build-all")]
    pub build_all: bool,

    #[arg(long = "javac-args", num_args = 1.., allow_hyphen_values = true)]
    pub javac_args: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct BuildCommand {
    #[command(subcommand)]
    pub command: Option<BuildSubCommand>,

    #[command(flatten)]
    pub args: BuildArgs,
}
#[derive(Subcommand, Debug, Clone)]
pub enum BuildSubCommand {
    /// Shows files that have been modified since last build
    Modified {},
    /// Shows all files and their dependancies
    Dependancies {},
    /// Shows all files and their dependants
    Dependants {},
    /// Shows all stale files will be recompiled next build
    Stale {},
}
#[derive(Debug, Parser, Clone)]
pub struct FindArgs {}

#[derive(Debug, Parser, Clone)]
pub struct CreateArgs {
    /// The name of the project being created
    #[arg(long, short)]
    pub name: Option<String>,

    /// Whether to initalize a git repository
    #[arg(long = "git", short = 'g')]
    pub init_git: Option<bool>,

    /// Dont initalize with example files
    #[arg(long = "bare", short = 'b')]
    pub bare: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct LazyJavaGlobalArgs {
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Where to find the java files to compile
    #[arg(long = "source", short = 's', default_value_t = SRC_FOLDER.to_string(), global = true)]
    pub source: String,

    /// Where to save the compiled java files
    #[arg(long = "bin", short = 'b', default_value_t = BUILD_FOLDER.to_string(), global = true)]
    pub build: String,

    /// Where to look for extra packages
    #[arg(long = "lib", short = 'l', default_value_t = LIB_FOLDER.to_string(), global = true)]
    pub lib: String,
}
