use std::{
    io, path,
    process::{ExitStatus, Output, Stdio},
};

use crate::{
    Context, JAVAC_SEPARATOR,
    args::RunArgs,
    build::BuildError,
    lock_file::LockFile,
    utils::{IOError, SeparatorList, processes::java_tool_command},
};

/// Build and run a raw `java` command. `classpath` is emitted only when
/// present (mirroring how [`crate::build::compile::compile_command`] handles
/// optional javac flags); `class` and `args` become positional arguments.
fn run_command(classpath: Option<&str>, class: &str, args: &[String]) -> Result<Output, io::Error> {
    let mut cmd = java_tool_command("java");
    if let Some(cp) = classpath {
        cmd.args(["-classpath", cp]);
    }
    cmd.arg(class).args(args);

    log::debug!("Running command {:?}", &cmd);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
}

/// Build and run a compiled Java class. The runtime classpath is derived from
/// the `Context`: the compiled `bin` output dir, every jar in `lib` (`lib/*`),
/// and any local dependency jars declared in the config.
pub fn execute_java(class: &str, args: &RunArgs, ctx: &Context) -> Result<ExitStatus, BuildError> {
    let ab_bin = path::absolute(&ctx.bin)
        .map_err(|e| BuildError::IoError(IOError::new("resolving classpath", &ctx.bin, e)))?;
    let ab_bin_processors = path::absolute(&ctx.bin_processors).map_err(|e| {
        BuildError::IoError(IOError::new(
            "resolving processor bin directory",
            &ctx.bin_processors,
            e,
        ))
    })?;

    let lockfile = LockFile::fetch(&ctx.root)?;

    let lib: Vec<String> = lockfile
        .runtime_packages()
        .into_iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    let classpath = SeparatorList::new(JAVAC_SEPARATOR)
        .add(ab_bin_processors.display())
        .add(ab_bin.display())
        .add_slice(&lib)
        .build();

    let output = run_command(Some(&classpath), class, &args.args).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            BuildError::JavaNotFound
        } else {
            BuildError::IoError(IOError::new("running java", &ab_bin, e))
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

    #[test]
    fn raw_run_command_resolves_java() -> Result<(), io::Error> {
        let out = super::run_command(None, "-version", &[])?.status;
        assert!(out.success());
        Ok(())
    }

    #[test]
    fn test_run_against_fixture() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_main_classes_test");
        current.push("build");
        let build = current.clone();
        let mut lib = current.clone();
        lib.pop();
        lib.push("lib");

        let sep = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };
        let classpath = format!("{}{sep}{}/*", build.display(), lib.display());

        let out = super::run_command(Some(&classpath), "Test2", &[])?;
        assert!(out.status.success(), "Run Command had a non-zero exit code");
        Ok(())
    }
}
