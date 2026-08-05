use std::error::Error;
use std::process::exit;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use lazy_java::{LazyJava, args::LazyJavaArgs, utils::DiagnosticProvider};
use log::LevelFilter;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();
    let verbose = args.global_args.verbose;

    let log_level = match verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::builder()
        .filter_level(log_level)
        .format_timestamp(None)
        .init();

    if let Err(e) = LazyJava::execute(args) {
        eprint!("{}", e.diagnostic());
        if verbose > 0 {
            print_causes(&e);
        }
        exit(1);
    }

    Ok(())
}

fn print_causes(e: &dyn Error) {
    let mut source = e.source();
    while let Some(cause) = source {
        eprintln!("{}: {}", "Caused by".yellow().bold(), cause);
        source = cause.source();
    }
}
