use std::fs;

use crate::{
    args::AddArgs,
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lsp::classpath::Classpath,
    maven_central::{
        MavenError, get_artifact_metadata, get_jar,
        pom::{DependancyType, MavenDependancyList},
    },
};

impl LazyJava {
    pub fn add(&self, add_args: &AddArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let version: Result<String, MavenError> = match &add_args.artifact_version {
            Some(version) => Ok(version.to_string()),
            None => {
                let meta = get_artifact_metadata(&add_args.group, &add_args.artifact)?;
                Ok(meta.versioning.release)
            }
        };

        let version = version?;

        let deps = MavenDependancyList::new(&add_args.group, &add_args.artifact, &version)?;

        // writing dependancy dependancies
        for dep in deps {
            if dep.dependancy_type != DependancyType::Jar {
                println!("Only jar dependancies are supported");
            }
            let jar = get_jar(&dep.group, &dep.artifact, &dep.version)?;

            let path = self
                .lib
                .join(format!("{}-{}.jar", &dep.artifact, &dep.version));

            fs::write(path, jar).map_err(|e| MavenError::UnableToWrite(e))?;
        }

        // writing the original dependancy
        let jar = get_jar(&add_args.group, &add_args.artifact, &version)?;

        let path = self
            .lib
            .join(format!("{}-{}.jar", &add_args.artifact, &version));

        fs::write(path, jar).map_err(|e| MavenError::UnableToWrite(e))?;

        Classpath::generate(self)?;

        Ok(())
    }
}
