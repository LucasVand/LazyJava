use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

pub mod args;
pub mod build;
pub mod clean;
pub mod create;
pub mod dependancy_graph;
pub mod find;
pub mod interactive;
pub mod lazy_java;
pub mod lazy_java_error;
pub mod logger;
pub mod lsp;
pub mod run;
pub mod utils;

pub const BUILD_FOLDER: &'static str = "bin";
pub const SRC_FOLDER: &'static str = "src";
pub const LIB_FOLDER: &'static str = "lib";

pub static IMPORT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return Regex::new(r"\s*import\s*(?<import>.*);").unwrap();
});
pub static PACKAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return RegexBuilder::new(r"^\s*package\s*(?<package>.*);")
        .unicode(true)
        .build()
        .unwrap();
});

pub static MAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    return RegexBuilder::new(r"public static void main(.*) \{(?<content>[\s\S]*)\}")
        .unicode(true)
        .multi_line(true)
        .build()
        .unwrap();
});
pub static CLASS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    let re = RegexBuilder::new(
        r#"^\s*(?:(?:public|static|abstract|final)\s+)*class\s+(?<class>\S*)\s+(?:extend.*)*\s*(?:implements.*)*\s*\{(?<content>[\s\S]*)\}"#,
    ).multi_line(true).unicode(true).build();
    return re.unwrap();
});
