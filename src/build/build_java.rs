use crate::args::BuildArgs;
use crate::lazy_java::LazyJava;

use crate::{build::build_error::BuildError, processes::compile_java};

impl LazyJava {
    pub fn build(&self, _args: &BuildArgs) -> Result<(), BuildError> {
        let compile_res =
            compile_java(&self.src, &self.build).map_err(|e| BuildError::IOError(e))?;
        if compile_res.success() {
            return Ok(());
        } else {
            return Err(BuildError::BuildErrors);
        }
    }
}
