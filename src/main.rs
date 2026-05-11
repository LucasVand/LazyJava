use LazyJava::{
    args::LazyJavaArgs,
    maven_central::{get_jar, metadata::get_artifact_metadata},
};
use anyhow::Result;
use clap::Parser;
use log::LevelFilter;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();

    let me = get_artifact_metadata("org.json", "json").unwrap();
    println!(
        "{}",
        get_jar("org.json", "json", &me.versioning.release)
            .unwrap()
            .len()
    );

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
