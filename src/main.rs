use std::env;

use LazyJava::args::LazyJavaCommand;
use LazyJava::build::build_java::build_java;
use LazyJava::clean::clean::clean;
use LazyJava::run::run_java::run_java;
use LazyJava::{args::LazyJavaArgs, find_root::find_root};
use anyhow::{Context, Result};
use clap::Parser;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();

    let current_dir = env::current_dir().context("Could not get current directory")?;

    let root = find_root(&current_dir).context("Could not find root")?;

    match args.command {
        LazyJavaCommand::Run { args: run_args } => run_java(&root, run_args, &args.global_args)?,
        LazyJavaCommand::Build { args: build_args } => {
            build_java(&root, build_args, &args.global_args)?
        }
        LazyJavaCommand::Clean {} => clean(&root, &args.global_args)?,
    }

    return Ok(());
}
