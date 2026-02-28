use anyhow::Result;

fn main() -> Result<()> {
    let lazy = LazyJava::lazy_java::LazyJava::new()?;
    lazy.execute()?;

    return Ok(());
}
