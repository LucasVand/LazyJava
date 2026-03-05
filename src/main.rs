use LazyJava::args::LazyJavaArgs;
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = LazyJavaArgs::parse();
    let lazy = LazyJava::lazy_java::LazyJava::new(args)?;
    lazy.execute()?;

    return Ok(());
}
