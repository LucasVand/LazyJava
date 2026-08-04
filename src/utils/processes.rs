use std::{
    env, io,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

use log::debug;

use crate::{
    JAVAC_SEPERATOR, build::BuildError, utils::IOError,
};


/// Build a command for a JDK tool (`javac`, `jar`, `java`). Prefers the
/// executable from `$JAVA_HOME/bin` and falls back to the system PATH.
pub fn java_tool_command(tool: &str) -> Command {
    if let Some(home) = env::var_os("JAVA_HOME") {
        let base = Path::new(&home).join("bin");
        let mut candidate = base.join(tool);
        if cfg!(windows) && candidate.extension().is_none() {
            candidate.set_extension("exe");
        }
        if candidate.is_file() {
            return Command::new(candidate);
        }
    }
    Command::new(tool)
}



fn run_command(
    build: &str,
    lib: &str,
    class: &str,
    args: &Vec<String>,
) -> Result<Output, io::Error> {
    let sep = JAVAC_SEPERATOR;

    let mut c = java_tool_command("java");
    let command = c
        .args(["-classpath", &format!("{}{sep}{}/*", build, lib)])
        .arg(class)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    debug!("Run Command {:?}", &command);

    let output = command.output()?;
    Ok(output)
}

pub fn execute_java(
    classname: &str,
    classpath: &PathBuf,
    lib: &Path,
    args: &Vec<String>,
) -> Result<ExitStatus, BuildError> {
    log::info!("Executing Java class: {}", classname);
    log::debug!("Using classpath: {:?}", classpath);
    log::debug!("Using library path: {:?}", lib);
    if !args.is_empty() {
        log::debug!("Program arguments: {:?}", args);
    }

    let ab_classpath = path::absolute(classpath)
        .map_err(|e| BuildError::IoError(IOError::new("resolving classpath", classpath, e)))?;
    let ab_lib = path::absolute(lib)
        .map_err(|e| BuildError::IoError(IOError::new("resolving library path", lib, e)))?;

    let output = run_command(
        ab_classpath.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        classname,
        args,
    )
    .map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            BuildError::JavaNotFound
        } else {
            BuildError::IoError(IOError::new("running java", &ab_classpath, e))
        }
    })?;

    if output.status.success() {
        log::info!("Java execution completed successfully");
    } else {
        log::warn!(
            "Java execution failed with exit code: {:?}",
            output.status.code()
        );
    }

    Ok(output.status)
}

#[cfg(test)]
mod tests {
    use std::{env, io};

    use crate::utils::processes::execute_java;

    #[test]
    fn test_run() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_main_classes_test");

        current.push("build");
        let build = current.clone();
        let mut lib = current.clone();
        lib.pop();
        lib.push("lib");

        let run = execute_java("Test2", &build, &lib, &Vec::new());

        assert!(run.is_ok(), "Run Command was an error");

        assert!(
            run.unwrap().success(),
            "Run Command had a none zero exit code"
        );

        return Ok(());
    }
}
