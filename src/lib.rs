use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

pub mod args;
pub mod build;
pub mod clean;
pub mod config;
pub mod create;
pub mod dependancy_graph;
pub mod find;
mod lazy_java;
pub mod lazy_java_error;
pub mod lock_file;
pub mod lsp;
pub mod maven_central;
pub mod packages;
pub mod run;
pub mod utils;

pub use lazy_java::LazyJava;

pub const BUILD_FOLDER: &str = "bin";
pub const SRC_FOLDER: &str = "src";
pub const LIB_FOLDER: &str = "lib";

pub const MAVEN_URL: &str = "https://repo1.maven.org/maven2/";

pub const LOCK_FILE_NAME: &str = "lazy-java.lock";
pub const CONFIG_FILE_NAME: &str = "lazy-java.toml";

pub fn create_maven_url(group: &str, artifact: &str) -> String {
    format!("{}{}/{}/", MAVEN_URL, group.replace(".", "/"), artifact)
}

pub static IMPORT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*import\s*(?<import>.*);").unwrap());
pub static PACKAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^\s*package\s*(?<package>.*);")
        .unicode(true)
        .build()
        .unwrap()
});

pub static MAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"public static void main(.*) \{(?<content>[\s\S]*)\}")
        .unicode(true)
        .multi_line(true)
        .build()
        .unwrap()
});
pub static CLASS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let re = RegexBuilder::new(
        r#"^\s*(?:(?:public|static|abstract|final)\s+)*class\s+(?<class>\S*)\s+(?:extend.*)*\s*(?:implements.*)*\s*\{(?<content>[\s\S]*)\}"#,
    ).multi_line(true).unicode(true).build();
    re.unwrap()
});
