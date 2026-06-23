use anyhow::Result;
use clap::Parser;
use lazy_java::{LazyJava, args::LazyJavaArgs};
use log::LevelFilter;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();

    let log_level = match args.global_args.verbose {
        0 => LevelFilter::Off,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    env_logger::builder()
        .filter_level(log_level)
        .format_timestamp_millis()
        .init();

    let lazy = LazyJava::new(args)?;
    lazy.execute()?;

    Ok(())
}
