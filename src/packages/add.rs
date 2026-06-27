use colored::Colorize;

use crate::{
    args::AddArgs,
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lock_file::LockFile,
    lsp::classpath::Classpath,
    maven_central::{MavenError, MavenIdBuf, fetch_artifact_metadata, pom::MavenDependancyList},
};

impl LazyJava {
    pub fn add(&self, add_args: &AddArgs) -> Result<(), LazyJavaError> {
        if add_args.dry_run {
            println!(
                "{} ({} will be made)",
                "--dry-run".green().bold(),
                "No persistent changes".red().bold(),
            )
        }
        self.assert_build_lib_src()?;

        let mut lockfile = LockFile::fetch(&self.root)?;

        let version: Result<String, MavenError> = match &add_args.artifact_version {
            Some(version) => Ok(version.to_string()),
            None => {
                let meta = fetch_artifact_metadata(&add_args.group, &add_args.artifact)?;
                Ok(meta.versioning.release)
            }
        };

        let version = version?;

        let id = MavenIdBuf::new(&add_args.group, &add_args.artifact, &version);
        println!("{} {} to dependency list", "Adding".green().bold(), id);
        let deps = MavenDependancyList::new(id.clone())?;
        let dep_count = deps.len();

        lockfile.add_packages(deps.into_iter().map(|v| v.into()).collect());
        println!(
            "    {} {} (+ {} transitive {})",
            "Added".green().bold(),
            id,
            dep_count,
            if dep_count != 1 {
                "dependencies"
            } else {
                "dependency"
            }
        );

        if !add_args.dry_run {
            lockfile.write(&self.root)?;
        }

        lockfile.validate_current_packages(&self.lib, add_args.dry_run)?;

        if !add_args.dry_run {
            Classpath::generate(self)?;
        }

        Ok(())
    }
}
