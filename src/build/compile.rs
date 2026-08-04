use std::{
    io,
    path::{self, Path, PathBuf},
    process::{ExitStatus, Output, Stdio},
};

use log::warn;

use crate::{
    Context, JAVAC_SEPERATOR,
    build::BuildError,
    utils::{IOError, join_directory, processes::java_tool_command},
};

/// Anything postfixed with list should be a : or ; seperated list depending on platform, expect
/// src_list that is a space seperated list
pub(crate) fn compile_command(
    src_list: &str,
    output_dir: &str,
    bin_dir: &str,
    lib_dir: &str,
    annotation_lib_list: &str,
    annotation_lib: &str,
    src_generated_dir: &str,
    build_processor_dir: &str,
    javac_args: &Vec<String>,
    release: Option<&str>,
) -> Result<Output, io::Error> {
    let sep = JAVAC_SEPERATOR;
    let classpath =
        format!("{build_processor_dir}{sep}{bin_dir}{sep}{annotation_lib}/*{sep}{lib_dir}/*");
    let processorpath = format!("{annotation_lib_list}{sep}{build_processor_dir}");

    let mut cmd = java_tool_command("javac");
    cmd.arg("-s")
        .arg(src_generated_dir)
        .arg("-processorpath")
        .arg(&processorpath)
        .arg("-classpath")
        .arg(&classpath)
        .arg("-d")
        .arg(output_dir);
    if let Some(version) = release {
        cmd.arg("--release").arg(version);
    }
    cmd.args(javac_args);
    for src in src_list.split_whitespace() {
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
    let ab_lib = resolve("resolving library directory", &ctx.lib)?;
    let ab_bin = resolve("resolving bin directory", &ctx.bin)?;
    let ab_annotation_lib = resolve(
        "resolving annotation library directory",
        &ctx.lib_annotations,
    )?;
    let ab_annotation_lib_list = join_directory(&ctx.lib_annotations, JAVAC_SEPERATOR);
    let ab_src_generated = resolve("resolving generated source directory", &ctx.src_generated)?;
    let src_generated_destructured = join_directory(&ab_src_generated, ' ');
    let ab_bin_processor = resolve("resolving processor bin directory", &ctx.bin_processors)?;
    log::debug!("Processor build output directory: {:?}", ab_bin_processor);
    log::debug!("Annotation library path: {:?}", ab_annotation_lib_list);

    let mut src_des = string_list.join(" ");
    if compile_generated_source {
        src_des.push(' ');
        src_des.push_str(&src_generated_destructured);
    }

    let output = match compile_command(
        &src_des,
        ab_dest.to_str().unwrap(),
        ab_bin.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        &ab_annotation_lib_list,
        ab_annotation_lib.to_str().unwrap(),
        ab_src_generated.to_str().unwrap(),
        ab_bin_processor.to_str().unwrap(),
        javac_args,
        release,
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
