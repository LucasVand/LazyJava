use std::{
    env, fs, io,
    time::{Duration, SystemTime},
};

use filetime::FileTime;
use thiserror::Error;

use crate::{
    args::CreateArgs,
    create::{
        init_git::git_init,
        interactive::{interactive_git_init_name, interactive_project_name},
    },
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    logger::logger::Logger,
};

impl LazyJava {
    pub fn create(&self, args: &CreateArgs) -> Result<(), LazyJavaError> {
        let name = match &args.name {
            Some(name) => name.clone(),
            None => interactive_project_name()?,
        };

        let git: bool = match args.init_git {
            Some(value) => value,
            None => interactive_git_init_name()?,
        };
        let mut project_dir =
            env::current_dir().map_err(|e| CreateProjectError::CurrrentDirectoryError(e))?;
        project_dir.push(&name);

        fs::create_dir(&project_dir).map_err(|_e| CreateProjectError::CreateDirectoryError)?;

        let mut build = project_dir.clone();
        build.push("bin");

        let mut lib = project_dir.clone();
        lib.push("lib");
        let mut src = project_dir.clone();
        src.push("src");

        fs::create_dir(&build).map_err(|_e| CreateProjectError::CreateDirectoryError)?;
        fs::create_dir(&src).map_err(|_e| CreateProjectError::CreateDirectoryError)?;
        fs::create_dir(&lib).map_err(|_e| CreateProjectError::CreateDirectoryError)?;

        if !args.bare {
            let mut uppercase_name = name.clone();
            let ch = uppercase_name.remove(0);
            uppercase_name.insert(0, ch.to_ascii_uppercase());

            let mut example = project_dir.clone();
            example.push(format!("src/{}.java", uppercase_name));

            fs::write(&example, example_class(uppercase_name))
                .map_err(|e| CreateProjectError::CreateFileError(e))?;

            filetime::set_file_mtime(
                &example,
                FileTime::from(SystemTime::now() + Duration::from_mins(1)),
            )
            .map_err(|e| CreateProjectError::CreateFileError(e))?;
        }

        if git {
            let status = git_init(&project_dir).map_err(|e| CreateProjectError::NoInit(e))?;

            if !status.success() {
                return Err(CreateProjectError::NoGit)?;
            }
        }

        Logger::log("");
        Logger::log("  now run");
        Logger::log("");
        Logger::log(&format!("   cd {}", name));
        Logger::log("   LazyJava run");
        Logger::log("");

        return Ok(());
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

fn example_class(name: String) -> String {
    return format!(
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
    );
}
