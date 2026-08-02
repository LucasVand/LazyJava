use std::{
    env,
    time::{Duration, SystemTime},
};

use filetime::FileTime;

use crate::{
    BUILD_FOLDER, Context, LIB_FOLDER, SRC_FOLDER,
    args::CreateArgs,
    config::ConfigTomlEdit,
    create::{
        init_git::git_init,
        interactive::{interactive_git_init_name, interactive_project_name},
    },
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lsp::sync_lsp_config,
    utils::{IOError, fs},
};

use super::CreateError;

impl LazyJava {
    pub fn create(
        args: &CreateArgs,
        all_args: &crate::args::LazyJavaArgs,
    ) -> Result<(), LazyJavaError> {
        let dry_run = all_args.global_args.dry_run;
        log::info!("Starting create operation");
        let name = match &args.name {
            Some(name) => name.clone(),
            None => interactive_project_name()?,
        };

        log::debug!("Creating project: {}", name);
        let git: bool = match args.init_git {
            Some(value) => value,
            None => interactive_git_init_name()?,
        };
        let mut project_dir = env::current_dir()
            .map_err(|e| CreateError::IoError(IOError::new("reading current directory", ".", e)))?;
        project_dir.push(&name);

        log::debug!("Project directory: {:?}", project_dir);

        if !dry_run {
            fs::create_dir(&project_dir).map_err(|_e| CreateError::CreateDirectoryError)?;

            let target = project_dir.join("target");
            let build = target.join(BUILD_FOLDER);
            let lib = target.join(LIB_FOLDER);
            let mut src = project_dir.clone();
            src.push(SRC_FOLDER);

            log::debug!("Creating subdirectories: bin, lib, src, target");
            fs::create_dir(&target).map_err(|_e| CreateError::CreateDirectoryError)?;
            fs::create_dir(&build).map_err(|_e| CreateError::CreateDirectoryError)?;
            fs::create_dir(&src).map_err(|_e| CreateError::CreateDirectoryError)?;
            fs::create_dir(&lib).map_err(|_e| CreateError::CreateDirectoryError)?;

            fs::write(
                project_dir.join("pom.xml"),
                "This file is to make sure root finders find this project (do not remove)",
            )
            .map_err(|e| {
                CreateError::IoError(IOError::new(
                    "writing pom.xml marker",
                    project_dir.join("pom.xml"),
                    e,
                ))
            })?;

            if !args.bare {
                log::debug!("Creating example Main class");
                let mut example = project_dir.clone();
                example.push(format!("src/{}.java", "Main"));

                fs::write(&example, example_class("Main")).map_err(|e| {
                    CreateError::IoError(IOError::new("writing example class", &example, e))
                })?;

                filetime::set_file_mtime(
                    &example,
                    FileTime::from(SystemTime::now() + Duration::from_mins(1)),
                )
                .map_err(|e| {
                    CreateError::IoError(IOError::new(
                        "setting file modification time",
                        &example,
                        e,
                    ))
                })?;
            }

            if git {
                log::debug!("Initializing git repository");
                let status = git_init(&project_dir).map_err(|e| {
                    CreateError::IoError(IOError::new("initializing git", &project_dir, e))
                })?;

                if !status.success() {
                    log::error!("Git initialization failed");
                    return Err(CreateError::NoGit)?;
                }
                log::debug!("Git repository initialized");
            }
        }

        let mut config = ConfigTomlEdit::parse("")?;
        let mut p = config.project_mut().get_or_insert_empty();
        p.name_mut().set(name.clone());

        if !dry_run {
            config.write(&project_dir)?;

            env::set_current_dir(&project_dir).map_err(|e| {
                CreateError::IoError(IOError::new("changing directory", &project_dir, e))
            })?;
            let ctx = Context::new_options(None, Some(config))?;
            sync_lsp_config(&ctx)?;
        }

        println!();
        println!("  now run");
        println!();
        println!("   cd {}", name);
        println!("   LazyJava run");
        println!();

        log::info!("Project created successfully: {}", name);
        Ok(())
    }
}

fn example_class(name: &str) -> String {
    format!(
        r#"
/* Created with LazyJava */ 
public class {} {{
        
    public static void main(String[] args) {{
        System.out.println("Hello world!");
        System.out.println("Welcome to your LazyJava project");
    }}
}}
"#,
        name
    )
}
