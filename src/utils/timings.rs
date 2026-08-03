use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use colored::Colorize;

pub struct Timings {
    title: String,
    pub total: Instant,
    current: Instant,
    steps: Vec<(String, Duration)>,
}

impl Timings {
    pub fn start(title: impl Into<String>) -> Self {
        Timings {
            title: title.into(),
            total: Instant::now(),
            current: Instant::now(),
            steps: Vec::new(),
        }
    }

    pub fn record(&mut self, name: impl Into<String>, duration: Duration) {
        self.steps.push((name.into(), duration));
    }

    pub fn record_current(&mut self, name: impl Into<String>) {
        self.steps.push((name.into(), self.current.elapsed()));
        self.current = Instant::now();
    }
}
impl Display for Timings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} timings", self.title.green().bold())?;
        for (desc, time) in &self.steps {
            writeln!(
                f,
                "  {:<36} {:>10.4}s",
                desc.white().bold(),
                time.as_secs_f64()
            )?;
        }
        writeln!(
            f,
            "{:<36} {:>10.4}s",
            "Total".green().bold(),
            self.total.elapsed().as_secs_f64()
        )
    }
}
