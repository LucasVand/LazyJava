use std::{
    fs, io,
    path::PathBuf,
    thread::sleep,
    time::{Duration, SystemTime},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessesToUpdate, System};
use thiserror::Error;

use crate::{ARTIFACT_LOCK_FILE, Context};

#[derive(Serialize, Deserialize, Debug)]
pub struct Lock {
    pub start: SystemTime,
    #[serde(with = "pid_serde")]
    pub pid: Pid,
}

mod pid_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sysinfo::Pid;

    pub fn serialize<S>(pid: &Pid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the Pid to a standard usize for transport
        pid.as_u32().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pid, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Read the standard usize value and wrap it back into Pid
        let raw = u32::deserialize(deserializer)?;
        Ok(Pid::from(raw as usize))
    }
}

impl Lock {
    pub fn acquire(ctx: &Context) -> Result<(), LockError> {
        let path = ctx.target.join(ARTIFACT_LOCK_FILE);
        if !path.try_exists()? {
            Self::create_lock(ctx)?;
            return Ok(());
        }
        let lock_str = fs::read_to_string(&path)?;

        let lock: Lock = toml::from_str(&lock_str)?;

        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        while sys.process(lock.pid).is_some() && path.try_exists()? {
            println!(
                "{} for lock on artifact form PID {}",
                "Waiting".red().bold(),
                lock.pid
            );
            sleep(Duration::from_secs(1));
            sys.refresh_processes(ProcessesToUpdate::All, true);
        }

        Self::create_lock(ctx)?;

        return Ok(());
    }
    fn create_lock(ctx: &Context) -> Result<(), LockError> {
        let lock = Lock {
            start: SystemTime::now(),
            pid: sysinfo::get_current_pid().expect("Unsupported platform"),
        };

        let s = toml::to_string_pretty(&lock)?;

        fs::write(ctx.target.join(ARTIFACT_LOCK_FILE), &s)?;
        Ok(())
    }

    pub fn release_lock(ctx: &Context) -> Result<(), LockError> {
        fs::remove_file(ctx.target.join(ARTIFACT_LOCK_FILE))?;
        return Ok(());
    }
}

#[derive(Error, Debug)]
pub enum LockError {
    #[error("IO operation failed")]
    IO(#[from] io::Error),

    #[error("Unable to deseralize lock")]
    DesError(#[from] toml::de::Error),

    #[error("Unable to seralize lock")]
    SerError(#[from] toml::ser::Error),
}
