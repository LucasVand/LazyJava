use std::{
    io,
    path::{self, PathBuf},
    process::{Command, ExitStatus, Stdio},
};
fn compile_command(src: &str, build: &str) -> String {
    if cfg!(target_os = "windows") {
        format!(
            r#"for /r {} %f in (*.java) do javac -d {} "%f""#,
            src, build
        )
    } else {
        format!(
            r#"find {} -name "*.java" -exec javac -d {} {{}} +"#,
            src, build
        )
    }
}

pub fn compile_java(src: &PathBuf, dest: &PathBuf) -> Result<ExitStatus, io::Error> {
    let ab_src = path::absolute(src)?;
    let ab_dest = path::absolute(dest)?;

    let command = compile_command(ab_src.to_str().unwrap(), ab_dest.to_str().unwrap());

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Compile Process Failed");

    return Ok(output.status);
}
pub fn execute_java(
    classname: &str,
    classpath: &PathBuf,
    args: &Vec<String>,
) -> Result<ExitStatus, io::Error> {
    let ab_classpath = path::absolute(classpath)?;
    let command = format!(
        "java -classpath {} {}",
        ab_classpath.to_str().unwrap(),
        classname,
    );
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .args(args.as_slice())
        .stdout(Stdio::inherit()) // Inherit the parent's stdout
        .stderr(Stdio::inherit()) // Inherit the parent's stderr
        .output()
        .expect("Run Command Failed");

    return Ok(output.status);
}

#[cfg(test)]
mod tests {
    use std::{env, io};

    use crate::utils::processes::{compile_java, execute_java};

    #[test]
    fn test_processes() -> Result<(), io::Error> {
        let mut current = env::current_dir()?;
        current.push("test_filesystem");
        current.push("find_main_classes_test");

        let src = current.clone();
        current.push("build");
        let build = current.clone();

        let comp = compile_java(&src, &build);
        let run = execute_java("Test1", &build, &Vec::new());

        assert!(comp.is_ok(), "Compile Command was an error");
        assert!(run.is_ok(), "Run Command was an error");

        return Ok(());
    }
}
