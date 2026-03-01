use crate::args::BuildArgs;
use crate::lazy_java::LazyJava;

use crate::lazy_java_error::LazyJavaError;
use crate::processes::compile_java;

impl LazyJava {
    pub fn build(&self, _args: &BuildArgs) -> Result<(), LazyJavaError> {
        let compile_res = compile_java(&self.src, &self.build)
            .map_err(|e| return LazyJavaError::UnableToCompile(e))?;
        if compile_res.success() {
            return Ok(());
        } else {
            return Err(LazyJavaError::CompilationErrors);
        }
    }
}
