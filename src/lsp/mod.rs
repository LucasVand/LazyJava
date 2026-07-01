pub mod classpath;
pub mod classpath_error;
pub mod classpath_impl;
mod project;

mod settings;
pub use project::DotProject;
pub use settings::DotSettings;

use crate::{Context, lazy_java_error::LazyJavaError, lsp::classpath::Classpath};

pub fn sync_lsp_config(ctx: &Context) -> Result<(), LazyJavaError> {
    Classpath::generate_if_stale(ctx)?;
    DotSettings::generate(&ctx.root)?;
    DotProject::generate(&ctx.root, &ctx.config)?;

    Ok(())
}
