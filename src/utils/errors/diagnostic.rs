use std::fmt::Display;

use colored::Colorize;

pub enum Level {
    Info,
    Warn,
    Error,
}

pub struct Diagnostic {
    title: String,
    message: Option<String>,
    help: Vec<String>,
    note: Vec<String>,
    level: Level,
}

pub trait DiagnosticProvider {
    fn diagnostic(&self) -> Diagnostic;
}
impl Diagnostic {
    pub fn new<S: Into<String>>(title: S) -> Diagnostic {
        Diagnostic {
            title: title.into(),
            message: None,
            help: Vec::new(),
            note: Vec::new(),
            level: Level::Error,
        }
    }
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
    pub fn help<S: Into<String>>(mut self, help: S) -> Self {
        self.help.push(help.into());
        self
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn note<S: Into<String>>(mut self, note: S) -> Self {
        self.note.push(note.into());
        self
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = match self.level {
            Level::Info => "Info".green().bold(),
            Level::Warn => "Warning".yellow().bold(),
            Level::Error => "Error".red().bold(),
        };
        writeln!(f, "{}: {}", w, self.title)?;

        if let Some(msg) = &self.message {
            writeln!(f, "{}", msg)?;
        }

        if !self.help.is_empty() || !self.note.is_empty() {
            writeln!(f)?;
        }

        for help in &self.help {
            writeln!(f, "{}: {}", "help".green().bold(), help)?;
        }
        for note in &self.note {
            writeln!(f, "{}: {}", "note".bold(), note)?;
        }

        std::fmt::Result::Ok(())
    }
}
