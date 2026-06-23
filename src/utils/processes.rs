use std::{
    io,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

fn compile_command(
    src: &str,
    build: &str,
    lib: &str,
    javac_args: &Vec<String>,
) -> Result<Output, io::Error> {
    let args = javac_args.join(" ");
    if cfg!(target_os = "windows") {
        let command = format!(
            r#"& {{javac -classpath "{}/*" -d "{}" {} (Get-ChildItem -Recurse -Filter *.java -Path "{}").FullName}}"#,
            lib, build, args, src
        );
        log::debug!("Windows javac command: {}", command);

        Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output()
    } else {
        let command = format!(
            r#"find {} -name "*.java" -exec javac -classpath "{}/*" -d "{}" {} {{}} +"#,
            src, lib, build, args
        );
        log::debug!("Unix javac command: {}", command);

        Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output()
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
            r#"&{{ javac -classpath "{};{}/*" -d "{}" {} {} }}"#,
            build, lib, build, args, files_str
        );

        log::debug!("Windows javac compile files command: {}", command);

        

        Command::new("powershell")
            .args(["-Command", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output()
    } else {
        let command = format!(
            r#"javac -classpath "{}:{}/*" -d "{}" {} {} "#,
            build, lib, build, args, files_str
        );

        log::debug!("Unix javac compile files command: {}", command);

        

        Command::new("sh")
            .args(["-c", &command])
            .stdout(Stdio::inherit()) // Inherit the parent's stdout
            .stderr(Stdio::inherit()) // Inherit the parent's stderr
            .output()
    }
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
    javac_args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    log::info!("Compiling Java from {:?} to {:?}", src, dest);
    log::debug!("Using library path: {:?}", lib);
    log::debug!("Javac arguments: {:?}", javac_args);

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
    build: &Path,
    lib: &Path,
    javac_args: &Vec<String>,
    files: Vec<PathBuf>,
) -> Result<ExitStatus, io::Error> {
    log::info!("Compiling {} Java file(s) to {:?}", files.len(), build);
    log::debug!("Using library path: {:?}", lib);
    log::debug!("Files to compile: {:?}", files);
    log::debug!("Javac arguments: {:?}", javac_args);

    let ab_build = path::absolute(build)?;
    let ab_lib = path::absolute(lib)?;

    let file_str: Vec<String> = files
        .into_iter()
        .map(|f| {
            format!(r#"{}"#, f.to_string_lossy())
        })
        .collect();

    let output = compile_files_command(
        ab_build.to_str().unwrap(),
        ab_lib.to_str().unwrap(),
        file_str,
        javac_args,
    )?;

    if output.status.success() {
        log::info!("File compilation completed successfully");
    } else {
        log::warn!(
            "File compilation failed with exit code: {:?}",
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

        let comp = compile_java(&src, &build, &lib, &Vec::new());

        assert!(comp.is_ok(), "Compile Command was an error");

        assert!(
            comp.unwrap().success(),
            "Compile Command had a none zero exit code"
        );

        return Ok(());
    }
}
