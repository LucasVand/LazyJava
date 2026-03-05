use std::{
    io,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

use crate::logger::logger::Logger;
fn compile_command(
    src: &str,
    build: &str,
    lib: &str,
    javac_args: &Vec<String>,
) -> Result<Output, io::Error> {
    let args = javac_args.join(" ");
    if cfg!(target_os = "windows") {
        let command = format!(
            r#"& {{javac -classpath "{}" -d "{}" {} (Get-ChildItem -Recurse -Filter *.java -Path "{}").FullName}}"#,
            lib, build, args, src
        );
        Logger::verbose_elog(&format!("javac command: {}", command));

        return Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output();
    } else {
        let command = format!(
            r#"find {} -name "*.java" -exec javac -classpath "{}" -d "{}" {} {{}} +"#,
            src, lib, build, args
        );
        Logger::verbose_elog(&format!("javac command: {}", command));

        return Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output();
    }
}
fn compile_files_command(
    build: &str,
    lib: &str,
    files: Vec<String>,
    javac_args: &Vec<String>,
) -> Result<Output, io::Error> {
    let files_str = files.join(" ");
    let args = javac_args.join(" ");
    if cfg!(target_os = "windows") {
        let command = format!(
            r#"&{{ javac -classpath "{};{}" -d "{}" {} {} }}"#,
            build, lib, build, args, files_str
        );

        Logger::verbose_elog(&format!("javac command: {}", command));

        let output = Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output();

        return output;
    } else {
        let command = format!(
            r#"javac -classpath "{}:{}" -d "{}" {} {} "#,
            build, lib, build, args, files_str
        );
        Logger::verbose_elog(&format!("javac command: {}", command));

        let output = Command::new("sh")
            .args(["-c", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output();

        return output;
    }
}
fn run_command(build: &str, class: &str, args: &Vec<String>) -> Result<Output, io::Error> {
    let args_str = args.join(" ");
    let command = format!(r#"java -classpath {} {} {}"#, build, class, args_str);
    if cfg!(target_os = "windows") {
        return Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output();
    } else {
        return Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output();
    }
}

pub fn compile_java(
    src: &Path,
    dest: &Path,
    lib: &Path,
    javac_args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    let ab_src = path::absolute(src)?;
    let ab_dest = path::absolute(dest)?;
    let ab_lib = path::absolute(lib)?;

    let command = compile_command(
        ab_src.to_str().unwrap(),
        ab_dest.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        javac_args,
    );

    let output = command.expect("Compile Command Failed");

    return Ok(output.status);
}

pub fn compile_java_files(
    build: &Path,
    lib: &Path,
    javac_args: &Vec<String>,
    files: Vec<PathBuf>,
) -> Result<ExitStatus, io::Error> {
    let ab_build = path::absolute(build)?;

    let ab_lib = path::absolute(lib)?;

    let file_str: Vec<String> = files
        .into_iter()
        .map(|f| {
            return format!(r#"{}"#, f.to_string_lossy());
        })
        .collect();

    let output = compile_files_command(
        ab_build.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        file_str,
        javac_args,
    )?;

    return Ok(output.status);
}
pub fn execute_java(
    classname: &str,
    classpath: &PathBuf,
    args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    let ab_classpath = path::absolute(classpath)?;

    let output =
        run_command(ab_classpath.to_str().unwrap(), classname, args).expect("Run Command Failed");

    return Ok(output.status);
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

        let run = execute_java("Test2", &build, &Vec::new());

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

        let comp = compile_java(&src, &build, &lib, &Vec::new());

        assert!(comp.is_ok(), "Compile Command was an error");

        assert!(
            comp.unwrap().success(),
            "Compile Command had a none zero exit code"
        );

        return Ok(());
    }
}
