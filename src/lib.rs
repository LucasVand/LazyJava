pub mod args;
pub mod build;
pub mod clean;
pub mod find;
pub mod find_main;
pub mod find_root;
pub mod lazy_java;
pub mod lazy_java_error;
pub mod processes;
pub mod run;
pub mod utils;

pub const BUILD_FOLDER: &'static str = "bin";
pub const SRC_FOLDER: &'static str = "src";
pub const LIB_FOLDER: &'static str = "lib";
