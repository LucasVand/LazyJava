use std::{
    env, fs, io,
    time::{Duration, SystemTime},
};

use filetime::FileTime;
use thiserror::Error;

use crate::{
    BUILD_FOLDER, Context, LIB_FOLDER, SRC_FOLDER,
    args::CreateArgs,
    config::Config,
    create::{
        init_git::git_init,
        interactive::{interactive_git_init_name, interactive_project_name},
    },
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lsp::sync_lsp_config,
};

impl LazyJava {
    pub fn create(args: &CreateArgs) -> Result<(), LazyJavaError> {
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
        let mut project_dir =
            env::current_dir().map_err(CreateProjectError::CurrrentDirectoryError)?;
        project_dir.push(&name);

        log::debug!("Project directory: {:?}", project_dir);
        fs::create_dir(&project_dir).map_err(|_e| CreateProjectError::CreateDirectoryError)?;

        let mut build = project_dir.clone();
        build.push(BUILD_FOLDER);

        let mut lib = project_dir.clone();
        lib.push(LIB_FOLDER);
        let mut src = project_dir.clone();
        src.push(SRC_FOLDER);

        log::debug!("Creating subdirectories: bin, lib, src");
        fs::create_dir(&project_dir.join("target"))
            .map_err(|_e| CreateProjectError::CreateDirectoryError)?;
        fs::create_dir(&build).map_err(|_e| CreateProjectError::CreateDirectoryError)?;

        fs::create_dir(&src).map_err(|_e| CreateProjectError::CreateDirectoryError)?;
        fs::create_dir(&lib).map_err(|_e| CreateProjectError::CreateDirectoryError)?;

        fs::write(
            &project_dir.join("pom.xml"),
            "This file is to make sure root finders find this project (do not remove)",
        )?;

        if !args.bare {
            log::debug!("Creating example Main class");
            let mut example = project_dir.clone();
            example.push(format!("src/{}.java", "Main"));

            fs::write(&example, example_class("Main"))
                .map_err(CreateProjectError::CreateFileError)?;

            filetime::set_file_mtime(
                &example,
                FileTime::from(SystemTime::now() + Duration::from_mins(1)),
            )
            .map_err(CreateProjectError::CreateFileError)?;
        }

        if git {
            log::debug!("Initializing git repository");
            let status = git_init(&project_dir).map_err(CreateProjectError::NoInit)?;

            if !status.success() {
                log::error!("Git initialization failed");
                return Err(CreateProjectError::NoGit)?;
            }
            log::debug!("Git repository initialized");
        }

        let mut config = Config::fetch(&project_dir)?;
        config.project.name = name.clone();

        config.write(&project_dir)?;

        env::set_current_dir(&project_dir)?;

        let ctx = Context::new_options(None, Some(config))?;

        sync_lsp_config(&ctx)?;

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
#[derive(Error, Debug)]
pub enum CreateProjectError {
    #[error("Couldnt prompt user for project name")]
    ProjectNameError,

    #[error("Couldnt create project directory")]
    CreateDirectoryError,

    #[error("Couldnt find current directory")]
    CurrrentDirectoryError(io::Error),

    #[error("Couldnt create files")]
    CreateFileError(io::Error),

    #[error("git is not install or included in path")]
    NoGit,

    #[error("Couldnt run git init, {0}")]
    NoInit(io::Error),
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
