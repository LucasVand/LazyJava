use std::{
    io,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

use log::warn;

use crate::{Context, JAVAC_SEPERATOR, utils::join_directory};

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
) -> Result<Output, io::Error> {
    let sep = JAVAC_SEPERATOR;
    let classpath =
        format!("{build_processor_dir}{sep}{bin_dir}{sep}{annotation_lib}/*{sep}{lib_dir}/*");
    let processorpath = format!("{annotation_lib_list}{sep}{build_processor_dir}");

    let mut cmd = Command::new("javac");
    cmd.arg("-s")
        .arg(src_generated_dir)
        .arg("-processorpath")
        .arg(&processorpath)
        .arg("-classpath")
        .arg(&classpath)
        .arg("-d")
        .arg(output_dir);
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
) -> Result<ExitStatus, io::Error> {
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

    let ab_dest = path::absolute(dest)?;
    let ab_lib = path::absolute(&ctx.lib)?;
    let ab_bin = path::absolute(&ctx.bin)?;
    let ab_annotation_lib = path::absolute(&ctx.lib_annotations)?;
    let ab_annotation_lib_list = join_directory(&ctx.lib_annotations, JAVAC_SEPERATOR);
    let ab_src_generated = path::absolute(&ctx.src_generated)?;
    let src_generated_destructured = join_directory(&ab_src_generated, ' ');
    let ab_bin_processor = path::absolute(&ctx.bin_processors)?;
    log::debug!("Processor build output directory: {:?}", ab_bin_processor);
    log::debug!("Annotation library path: {:?}", ab_annotation_lib_list);

    let mut src_des = string_list.join(" ");
    if compile_generated_source {
        src_des.push(' ');
        src_des.push_str(&src_generated_destructured);
    }

    let command = compile_command(
        &src_des,
        ab_dest.to_str().unwrap(),
        ab_bin.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        &ab_annotation_lib_list,
        ab_annotation_lib.to_str().unwrap(),
        ab_src_generated.to_str().unwrap(),
        ab_bin_processor.to_str().unwrap(),
        javac_args,
    );

    let output = command.expect("Compile Command Failed");

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
