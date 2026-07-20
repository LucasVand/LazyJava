use std::{
    io,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
};

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
