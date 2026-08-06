use super::compile::compile_command;
use std::fs;
use std::io::Write;

fn test_java_file(dir: &std::path::Path) -> std::path::PathBuf {
    let file_path = dir.join("Hello.java");
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(
        file,
        "public class Hello {{
    public static void main(String[] args) {{
        System.out.println(\"Hello, World!\");
    }}
}}"
    )
    .unwrap();
    file_path
}

#[test]
fn test_compile_command_simple() -> Result<(), std::io::Error> {
    let tmp = tempfile::tempdir()?;
    let src_dir = tmp.path().join("src");
    let build_dir = tmp.path().join("build");
    let lib_dir = tmp.path().join("lib");
    let annotation_lib_dir = tmp.path().join("lib-annotations");
    let src_generated_dir = tmp.path().join("generated-source");
    let processor_dir = tmp.path().join("processor-bin");

    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&build_dir)?;
    fs::create_dir_all(&lib_dir)?;
    fs::create_dir_all(&annotation_lib_dir)?;
    fs::create_dir_all(&src_generated_dir)?;
    fs::create_dir_all(&processor_dir)?;

    let java_file = test_java_file(&src_dir);

    let sep = crate::JAVAC_SEPERATOR;
    let classpath = format!(
        "{}{sep}{}{sep}{}/*{sep}{}/*",
        processor_dir.display(),
        build_dir.display(),
        annotation_lib_dir.display(),
        lib_dir.display()
    );
    let processorpath = format!(
        "{}{sep}{}",
        annotation_lib_dir.display(),
        processor_dir.display()
    );
    let source_files = vec![java_file.to_string_lossy().to_string()];

    let result = compile_command(
        Some(&build_dir.to_string_lossy()),
        Some(&classpath),
        Some(&processorpath),
        Some(&src_generated_dir.to_string_lossy()),
        None,
        &Vec::new(),
        &source_files,
    )?;

    assert!(
        result.status.success(),
        "Compile command failed with exit code: {:?}",
        result.status.code()
    );

    assert!(
        build_dir.join("Hello.class").exists(),
        "Expected Hello.class to exist"
    );

    Ok(())
}

#[test]
fn test_compile_command_with_package() -> Result<(), std::io::Error> {
    let tmp = tempfile::tempdir()?;
    let pkg_dir = tmp.path().join("src").join("com").join("example");
    let build_dir = tmp.path().join("build");
    let lib_dir = tmp.path().join("lib");
    let annotation_lib_dir = tmp.path().join("lib-annotations");
    let src_generated_dir = tmp.path().join("generated-source");
    let processor_dir = tmp.path().join("processor-bin");

    fs::create_dir_all(&pkg_dir)?;
    fs::create_dir_all(&build_dir)?;
    fs::create_dir_all(&lib_dir)?;
    fs::create_dir_all(&annotation_lib_dir)?;
    fs::create_dir_all(&src_generated_dir)?;
    fs::create_dir_all(&processor_dir)?;

    let file_path = pkg_dir.join("Greeting.java");
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(
        file,
        "package com.example;
public class Greeting {{
    public String greet() {{ return \"hi\"; }}
}}"
    )
    .unwrap();

    let sep = crate::JAVAC_SEPERATOR;
    let classpath = format!(
        "{}{sep}{}{sep}{}/*{sep}{}/*",
        processor_dir.display(),
        build_dir.display(),
        annotation_lib_dir.display(),
        lib_dir.display()
    );
    let processorpath = format!(
        "{}{sep}{}",
        annotation_lib_dir.display(),
        processor_dir.display()
    );
    let source_files = vec![file_path.to_string_lossy().to_string()];

    let result = compile_command(
        Some(&build_dir.to_string_lossy()),
        Some(&classpath),
        Some(&processorpath),
        Some(&src_generated_dir.to_string_lossy()),
        None,
        &Vec::new(),
        &source_files,
    )?;

    assert!(
        result.status.success(),
        "Package compile command failed with exit code: {:?}",
        result.status.code()
    );

    assert!(
        build_dir
            .join("com")
            .join("example")
            .join("Greeting.class")
            .exists(),
        "Expected Greeting.class to exist"
    );

    Ok(())
}
