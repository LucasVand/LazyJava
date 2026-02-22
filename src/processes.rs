use std::{
    io,
    path::{self, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

pub fn compile_java(src: &PathBuf, dest: &PathBuf) -> Result<ExitStatus, io::Error> {
    let ab_src = path::absolute(src)?;
    let ab_dest = path::absolute(dest)?;

    let command = format!(
        r#"find {} -name "*.java" -exec javac -d {} {{}} +"#,
        ab_src.to_str().unwrap(),
        ab_dest.to_str().unwrap()
    );
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
