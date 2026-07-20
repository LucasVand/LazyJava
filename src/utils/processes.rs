use std::{
    io,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

use crate::{
    JAVAC_SEPERATOR,
    utils::destructure_dir::{find_all_java_files, join_directory},
};
fn compile_command(
    src_list: &str,
    build_dir: &str,
    lib_dir: &str,
    annotation_lib_list: &str,
    src_generated_dir: &str,
    src_generated_list: &str,
    javac_args: &Vec<String>,
) -> Result<Output, io::Error> {
    let args = javac_args.join(" ");
    let sep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    let replace_sep = |input: &str, sep: &str| {
        return input.replace(sep, " ");
    };

    let source_list_replaced = replace_sep(src_generated_list, sep);
    let command = format!(
        r#"javac -s "{src_generated_dir}" -processorpath "{annotation_lib_list}" -classpath "{annotation_lib_list}{sep}{lib_dir}/*" -d "{build_dir}" {args} {src_list} {source_list_replaced}"#
    );

    log::info!("Compile Command: {}", command);

    let command = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    };

    return command;
}

fn run_command(
    build: &str,
    lib: &str,
    class: &str,
    args: &Vec<String>,
) -> Result<Output, io::Error> {
    let args_str = args.join(" ");
    if cfg!(target_os = "windows") {
        let command = format!(
            r#"java -classpath "{};{}/*" {} {}"#,
            build, lib, class, args_str
        );
        log::debug!("Windows java run command: {}", command);
        Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    } else {
        let command = format!(
            r#"java -classpath "{}:{}/*" {} {}"#,
            build, lib, class, args_str
        );
        log::debug!("Unix java run command: {}", command);
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    }
}

pub fn compile_java(
    src: &Path,
    dest: &Path,
    lib: &Path,
    annotation_lib: &Path,
    src_generated: &Path,
    javac_args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    log::info!("Compiling Java from {:?} to {:?}", src, dest);
    log::debug!("Using library path: {:?}", lib);
    log::debug!("Javac arguments: {:?}", javac_args);

    let src_des = find_all_java_files(&src);
    let ab_dest = path::absolute(dest)?;
    let ab_lib = path::absolute(lib)?;
    let ab_annotation_lib = join_directory(annotation_lib, JAVAC_SEPERATOR);
    let src_generated = path::absolute(src_generated)?;
    let src_generated_destructured = join_directory(&src_generated, JAVAC_SEPERATOR);

    let command = compile_command(
        &src_des,
        ab_dest.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        &ab_annotation_lib,
        src_generated.to_str().unwrap(),
        &src_generated_destructured,
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

pub fn compile_java_files(
    files: Vec<PathBuf>,
    dest: &Path,
    lib: &Path,
    annotation_lib: &Path,
    src_generated: &Path,
    javac_args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    log::info!("Compiling Java to to {:?}", dest);
    log::debug!("Using library path: {:?}", lib);
    log::debug!("Javac arguments: {:?}", javac_args);

    let src_des: String = files
        .into_iter()
        .filter_map(|f| {
            return path::absolute(f).ok();
        })
        .map(|a| a.to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    let ab_dest = path::absolute(dest)?;
    let ab_lib = path::absolute(lib)?;
    let ab_annotation_lib = join_directory(annotation_lib, JAVAC_SEPERATOR);
    let src_generated = path::absolute(src_generated)?;
    let src_generated_destructured = join_directory(&src_generated, JAVAC_SEPERATOR);

    let command = compile_command(
        src_des.as_str(),
        ab_dest.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        &ab_annotation_lib,
        src_generated.to_str().unwrap(),
        &src_generated_destructured,
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
pub fn execute_java(
    classname: &str,
    classpath: &PathBuf,
    lib: &Path,
    args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    log::info!("Executing Java class: {}", classname);
    log::debug!("Using classpath: {:?}", classpath);
    log::debug!("Using library path: {:?}", lib);
    if !args.is_empty() {
        log::debug!("Program arguments: {:?}", args);
    }

    let ab_classpath = path::absolute(classpath)?;
    let ab_lib = path::absolute(lib)?;

    let output = run_command(
        ab_classpath.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        classname,
        args,
    )
    .expect("Run Command Failed");

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

pub fn java_version() -> Result<String, io::Error> {
    let command = "java --version";
    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-Command", command])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
    }?;

    let str = String::from_utf8_lossy(&output.stdout);
    let mut split = str.split(" ");

    let version = split.next().unwrap();

    Ok(version.to_string())
}

#[cfg(test)]
mod tests {
    use std::{env, io};

    use crate::utils::processes::{compile_java, execute_java};

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

    #[test]
    fn test_compile() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_main_classes_test");

        let src = current.clone();
        current.push("build");
        let build = current.clone();
        let mut lib = current.clone();
        lib.pop();
        lib.push("lib");

        let comp = compile_java(
            &src,
            &build,
            &lib,
            &build.join("lib-annotations"),
            &build.join("src-generated"),
            &Vec::new(),
        );

        assert!(comp.is_ok(), "Compile Command was an error");

        assert!(
            comp.unwrap().success(),
            "Compile Command had a none zero exit code"
        );

        return Ok(());
    }
}
