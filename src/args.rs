use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[derive(Debug, Clone)]
pub struct LazyJavaArgs {
    /// Operation to execute
    #[command(subcommand)]
    pub command: LazyJavaCommand,

    #[command(flatten)]
    pub global_args: LazyJavaGlobalArgs,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LazyJavaCommand {
    /// Compile and run a java main class
    Run {
        #[command(flatten)]
        args: RunArgs,
    },
    /// Compile a java project
    Build {
        #[command(flatten)]
        args: BuildCommand,
    },
    /// Clean the java build folder
    Clean {},
    /// Finds all main classes and prints them
    Find {
        #[command(flatten)]
        args: FindArgs,
    },
    /// Creates a new java project
    Create {
        #[command(flatten)]
        args: CreateArgs,
    },
    /// Adds a new dependancy
    Add {
        #[command(flatten)]
        args: AddArgs,
    },
    /// Removes a dependancy
    Remove {
        #[command(flatten)]
        args: RemoveArgs,
    },
    /// Sync the lib folder with the lock file
    Sync {
        #[command(flatten)]
        args: SyncArgs,
    },
    Generate {
        #[command(flatten)]
        args: GenerateArgs,
    },
    Import {
        #[command(flatten)]
        args: ImportArgs,
    },
}
#[derive(Debug, Parser, Clone)]
pub struct RunArgs {
    /// The main class to run
    pub class: Option<String>,

    /// the args to pass to the program
    pub args: Vec<String>,

    /// Skip the compile step and run from build folder
    #[arg(long = "no-build", short = 'n')]
    pub no_build: bool,

    /// Run using the pre-built jar instead of classpath
    #[arg(long = "jar", short = 'j')]
    pub jar: bool,

    // #[arg(long = "args", short = 'a', num_args = 1.., allow_hyphen_values = true)]
    // pub args: Vec<String>,
    #[command(flatten)]
    pub build_args: BuildArgs,
}

#[derive(Debug, Parser, Clone)]
pub struct BuildArgs {
    /// Rebuild all files
    #[arg(long = "build-all")]
    pub build_all: bool,

    /// Show a per-step breakdown of compile times
    #[arg(long = "timings")]
    pub timings: bool,

    /// Shows all the files that were recompiled
    #[arg(long = "show-compiled")]
    pub show_compiled: bool,

    #[arg(long = "javac-args", num_args = 1.., allow_hyphen_values = true)]
    pub javac_args: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct BuildCommand {
    #[command(subcommand)]
    pub command: Option<BuildSubCommand>,

    #[command(flatten)]
    pub args: BuildArgs,
}
#[derive(Subcommand, Debug, Clone)]
pub enum BuildSubCommand {
    /// Shows all files and their dependancies
    Dependancies {},
    /// Shows all files and their dependants
    Dependants {},
    /// Shows all stale files will be recompiled next build
    Stale {},
    /// Rebuilds the .classfile which is used for jdtls
    Classpath {},
    /// Create a jar file
    Jar {
        #[command(flatten)]
        args: JarArgs,
    },
}

#[derive(Debug, Parser, Clone)]
pub struct JarArgs {
    /// The main entry point of the jar as a package path
    #[arg(long)]
    pub entry_point: Option<String>,

    /// Should create a fat jar with all dependancies bundled
    #[arg(short, long)]
    pub fat: bool,

    #[command(flatten)]
    pub build_args: BuildArgs,
}
#[derive(Debug, Parser, Clone)]
pub struct FindArgs {}

#[derive(Debug, Parser, Clone)]
pub struct CreateArgs {
    /// The name of the project being created
    #[arg(long, short)]
    pub name: Option<String>,

    /// Whether to initalize a git repository
    #[arg(long = "git", short = 'g')]
    pub init_git: Option<bool>,

    /// Dont initalize with example files
    #[arg(long = "bare", short = 'b')]
    pub bare: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct AddArgs {
    /// the name of the group which the artifact belongs to
    pub group: String,
    /// the name of the artifact
    pub artifact: String,
    // the specific version to add
    pub artifact_version: Option<String>,
}

#[derive(Debug, Parser, Clone)]
pub struct RemoveArgs {
    /// the name of the group which the artifact belongs to
    pub group: String,
    /// the name of the artifact
    pub artifact: String,
}

#[derive(Debug, Parser, Clone)]
pub struct SyncArgs {}

#[derive(Debug, Parser, Clone)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: GenerateCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GenerateCommand {
    Pom,
}

#[derive(Debug, Parser, Clone)]
pub struct ImportArgs {
    #[command(subcommand)]
    pub command: ImportCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ImportCommand {
    Pom {
        #[command(flatten)]
        args: ImportPomArgs,
    },
}
#[derive(Parser, Debug, Clone)]
pub struct ImportPomArgs {
    /// Path to the pom.xml file to import
    #[arg(long = "pom-path", default_value = "pom.xml")]
    pub pom_path: String,

    /// Overwrite existing lazy-java.toml if it exists
    #[arg(long = "overwrite")]
    pub overwrite: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct LazyJavaGlobalArgs {
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Where to find the java files to compile
    #[arg(long = "source", global = true)]
    pub source: Option<String>,

    /// Where to look for all build and config info
    #[arg(long = "target", global = true)]
    pub target: Option<String>,

    /// Show what would change without actually doing it
    #[arg(long = "dry-run", global = true)]
    pub dry_run: bool,
}
