use std::sync::OnceLock;

use colored::Colorize;

use crate::{Context, args::BuildArgs, utils::processes::java_tool_command};

/// Default JDK release used when the installed Java version cannot be detected.
pub const DEFAULT_RELEASE: &str = "25";

static INSTALLED_VERSION: OnceLock<String> = OnceLock::new();

/// The desired JDK release version for the build.
///
/// Resolution order:
/// 1. `--release` from CLI [`BuildArgs`]
/// 2. `[setup] release` from the project's config
/// 3. The major version of the installed JDK (via `java -version`), resolved
///    once and cached.
/// 4. [`DEFAULT_RELEASE`] with a warning, if detection fails.
pub fn desired_jdk_version(args: Option<&BuildArgs>, ctx: Option<&Context>) -> String {
    if let Some(args) = args
        && let Some(version) = &args.release
    {
        return version.trim().to_string();
    }

    if let Some(ctx) = ctx
        && let Some(setup) = ctx.config.setup()
        && let Some(version) = setup.release()
    {
        return version;
    }

    installed_jdk_version()
}

/// The major version of the installed JDK, resolved once and cached.
pub fn installed_jdk_version() -> String {
    INSTALLED_VERSION
        .get_or_init(detect_installed_version)
        .clone()
}

/// Warn if the installed JDK is older than the release the project was
/// compiled for, since the compiled classes may then fail to run.
pub fn warn_runtime_mismatch(release: &str) {
    let runtime = installed_jdk_version();
    match (runtime.parse::<i64>(), release.trim().parse::<i64>()) {
        (Ok(runtime), Ok(release)) if runtime < release => {
            println!(
                "{}: The installed JDK is {runtime} but the project targets JDK {release}. \
                 Compiled classes may not run; install a JDK >= {release}.",
                "Warning".yellow().bold()
            );
        }
        _ => {}
    }
}

fn detect_installed_version() -> String {
    let output = match java_tool_command("java").arg("-version").output() {
        Ok(output) => output,
        Err(e) => {
            warn_and_default(&format!("could not run java -version: {e}"));
            return DEFAULT_RELEASE.to_string();
        }
    };

    let raw = String::from_utf8_lossy(&output.stderr);
    let raw = raw.trim();
    log::debug!("java -version output: {}", raw);

    match parse_major(raw) {
        Some(major) => major,
        None => {
            warn_and_default(&format!("could not parse java version from: {raw}"));
            DEFAULT_RELEASE.to_string()
        }
    }
}

fn warn_and_default(reason: &str) {
    println!(
        "{}: {reason}; defaulting to JDK release {DEFAULT_RELEASE}. \
         Set `--release` or `[setup] release` to use a specific version.",
        "Warning".yellow().bold()
    );
}

/// Extract the major version from `java -version` output. Handles both the
/// modern (`"21.0.1"`) and legacy (`"1.8.0_432"`) formats.
fn parse_major(raw: &str) -> Option<String> {
    let opening = raw.find('"')?;
    let after = &raw[opening + 1..];
    let closing = after.find('"')?;
    let version = &after[..closing];

    let major = if let Some(legacy) = version.strip_prefix("1.") {
        legacy.split('.').next().unwrap_or(version)
    } else {
        version.split('.').next().unwrap_or(version)
    };

    Some(major.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_major;

    #[test]
    fn parses_modern_version() {
        assert_eq!(
            parse_major("openjdk version \"21.0.2\" 2026-04-21 LTS"),
            Some("21".into())
        );
    }

    #[test]
    fn parses_legacy_jdk8() {
        assert_eq!(parse_major("java version \"1.8.0_432\""), Some("8".into()));
    }

    #[test]
    fn returns_none_when_no_version() {
        assert_eq!(parse_major("javac something"), None);
    }
}
