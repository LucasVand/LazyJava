use std::{
    env,
    path::Path,
    process::Command,
};

/// Build a command for a JDK tool (`javac`, `jar`, `java`). Prefers the
/// executable from `$JAVA_HOME/bin` and falls back to the system PATH.
pub fn java_tool_command(tool: &str) -> Command {
    if let Some(home) = env::var_os("JAVA_HOME") {
        let base = Path::new(&home).join("bin");
        let mut candidate = base.join(tool);
        if cfg!(windows) && candidate.extension().is_none() {
            candidate.set_extension("exe");
        }
        if candidate.is_file() {
            return Command::new(candidate);
        }
    }
    Command::new(tool)
}
