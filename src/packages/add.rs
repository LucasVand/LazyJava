use crate::{
    args::AddArgs,
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    lock_file::LockFile,
    lsp::classpath::Classpath,
    maven_central::{MavenError, MavenId, get_artifact_metadata, pom::MavenDependancyList},
};

impl LazyJava {
    pub fn add(&self, add_args: &AddArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let mut lockfile = LockFile::fetch(&self.root)?;

        let version: Result<String, MavenError> = match &add_args.artifact_version {
            Some(version) => Ok(version.to_string()),
            None => {
                let meta = get_artifact_metadata(&add_args.group, &add_args.artifact)?;
                Ok(meta.versioning.release)
            }
        };

        let version = version?;

        let id = MavenId::new(&add_args.group, &add_args.artifact, &version);
        let deps = MavenDependancyList::new(&id)?;

        lockfile.add_packages(deps.into_iter().map(|v| v.into()).collect());

        lockfile.write(&self.root)?;

        lockfile.validate_current_packages(&self.lib)?;

        Classpath::generate(self)?;

        Ok(())
    }
}
