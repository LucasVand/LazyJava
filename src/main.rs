use LazyJava::args::LazyJavaArgs;
use anyhow::Result;
use clap::Parser;
use log::LevelFilter;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();
    
    let log_level = match args.global_args.verbose {
        0 => LevelFilter::Off,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    
    simple_logger::SimpleLogger::new()
        .with_level(log_level)
        .env()
        .init()
        .unwrap_or_default();
    
    let lazy = LazyJava::lazy_java::LazyJava::new(args)?;
    lazy.execute()?;

    return Ok(());
}
