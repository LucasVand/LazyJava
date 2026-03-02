use crate::args::BuildArgs;
use crate::lazy_java::LazyJava;

use crate::lazy_java_error::LazyJavaError;
use crate::logger::logger::Logger;
use crate::utils::processes::compile_java;

impl LazyJava {
    pub fn build(&self, _args: &BuildArgs) -> Result<(), LazyJavaError> {
        let compile_res = compile_java(&self.src, &self.build)
            .map_err(|e| return LazyJavaError::UnableToCompile(e))?;
        Logger::verbose_elog("Compiled Java");
        if compile_res.success() {
            Logger::verbose_elog("Compilation Exit Code Successful");
            return Ok(());
        } else {
            Logger::verbose_elog("Compilation Exit Code Not Successful");
            return Err(LazyJavaError::CompilationErrors);
        }
    }
}
