use std::{
    io,
    path::{self, Path, PathBuf},
    process::{ExitStatus, Output, Stdio},
};

use log::warn;

use crate::{
    Context, JAVAC_SEPARATOR,
    build::BuildError,
    lock_file::LockFile,
    utils::{IOError, SeparatorList, join_directory, processes::java_tool_command},
};

/// Build and run a `javac` command. Each `Option<&str>` maps to a javac flag
/// that is emitted only when present:
///
///   - `output_dir`         -> `-d <dir>`
///   - `classpath`          -> `-classpath <cp>`
///   - `processorpath`      -> `-processorpath <pp>`
///   - `src_generated_dir`  -> `-s <dir>`
///   - `release`            -> `--release <version>`
///
/// `javac_args` are forwarded verbatim and `source_files` are added as
/// positional arguments.
pub(crate) fn compile_command(
    output_dir: Option<&str>,
    classpath: Option<&str>,
    processorpath: Option<&str>,
    src_generated_dir: Option<&str>,
    release: Option<&str>,
    javac_args: &[String],
    source_files: &[String],
) -> Result<Output, io::Error> {
    let mut cmd = java_tool_command("javac");

    if let Some(dir) = output_dir {
        cmd.arg("-d").arg(dir);
    }
    if let Some(cp) = classpath {
        cmd.arg("-classpath").arg(cp);
    }
    if let Some(pp) = processorpath {
        cmd.arg("-processorpath").arg(pp);
    }
    if let Some(gen_dir) = src_generated_dir {
        cmd.arg("-s").arg(gen_dir);
    }
    if let Some(version) = release {
        cmd.arg("--release").arg(version);
    }
    cmd.args(javac_args);
    for src in source_files {
        cmd.arg(src);
    }

    log::debug!("Compile Command: {:?}", &cmd);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
}

pub fn compile_java(
    src_files: Vec<PathBuf>,
    dest: &Path,
    ctx: &Context,
    javac_args: &Vec<String>,
    compile_generated_source: bool,
    release: Option<&str>,
) -> Result<ExitStatus, BuildError> {
    log::debug!("Using library path: {:?}", ctx.lib);
    log::debug!("Javac arguments: {:?}", javac_args);

    let string_list: Vec<String> = src_files
        .into_iter()
        .filter_map(|path| {
            let con = path::absolute(path);
            if con.is_err() {
                warn!("Unable to absolute path")
            };
            con.ok()
        })
        .map(|c_path| c_path.to_string_lossy().to_string())
        .collect();

    let resolve = |what: &'static str, p: &Path| -> Result<PathBuf, BuildError> {
        path::absolute(p).map_err(|e| BuildError::IoError(IOError::new(what, p, e)))
    };

    let ab_dest = resolve("resolving output directory", dest)?;
    let ab_bin = resolve("resolving bin directory", &ctx.bin)?;

    let ab_src_generated = resolve("resolving generated source directory", &ctx.src_generated)?;
    let src_generated_destructured = join_directory(&ab_src_generated, ' ');
    let ab_bin_processor = resolve("resolving processor bin directory", &ctx.bin_processors)?;

    let lock_file = LockFile::fetch(&ctx.root)?;

    let compile_lib: Vec<String> = lock_file
        .compile_time_packages()
        .into_iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    let processor_lib: Vec<String> = lock_file
        .processor_compile_time_packages()
        .into_iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    let sep = JAVAC_SEPARATOR;
    let classpath = SeparatorList::new(sep)
        .add(ab_bin_processor.display())
        .add(ab_bin.display())
        .add_slice(&compile_lib)
        .build();

    let processorpath = SeparatorList::new(sep)
        .add_slice(&processor_lib)
        .add(ab_bin_processor.display())
        .build();

    let mut source_files = string_list;
    if compile_generated_source {
        source_files.extend(
            src_generated_destructured
                .split_whitespace()
                .map(|s| s.to_string()),
        );
    }

    let output = match compile_command(
        Some(ab_dest.to_str().unwrap()),
        Some(&classpath),
        Some(&processorpath),
        Some(ab_src_generated.to_str().unwrap()),
        release,
        javac_args,
        &source_files,
    ) {
        Ok(output) => output,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(BuildError::JavacNotFound),
        Err(e) => {
            return Err(BuildError::IoError(IOError::new(
                "running javac",
                &ab_bin,
                e,
            )));
        }
    };

    if output.status.success() {
        log::info!("Compilation completed successfully");
    } else {
        log::warn!(
            "Compilation failed with exit code: {:?}",
            output.status.code()
        );
    }

    Ok(output.status)
}
