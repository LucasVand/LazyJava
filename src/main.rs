use anyhow::Result;
use clap::Parser;
use lazy_java::{LazyJava, args::LazyJavaArgs};
use log::LevelFilter;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();

    let log_level = match args.global_args.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::builder()
        .format_target(cfg!(debug_assertions))
        .filter_level(log_level)
        .format_timestamp(None)
        .init();

    LazyJava::execute(args)?;

    Ok(())
}
