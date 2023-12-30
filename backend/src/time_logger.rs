use std::fs::{File, OpenOptions,};
use std::path::Path;
use std::io::{Error, ErrorKind,};

use chrono::Duration;
use anyhow::Result;

use csv::Writer;

use crate::Task;
use crate::utils::{today_string, now_string};

#[derive(Debug)]
pub struct TimeSheet {
    writer: Writer<File>,
}

impl TimeSheet {
    /// Impure (writes to file)
    /// # Errors
    /// Returns an error if:
    /// 1. A file already exists at the passed path
    /// 2. A file cannot be created at the passed path, or
    /// 3. The headers cannot be writen to the opened file
    pub fn new_timesheet(path: &str) -> Result<Self> {
        if Path::exists(Path::new(path)) {
            return Err(Error::new(ErrorKind::AlreadyExists, format!("Cannot create a logfile at: {path}\nFile exists!")).into());
        }

        let mut logger = Self {writer:Writer::from_path(path)?};

        logger.writer.write_record(["Task", "TaskID", "Time Worked", "Date", "Time"])?;

        Ok(logger)
    }

    /// Impure (opens file)
    /// # Errors
    /// Returns an error if the path cannot be opened for writing
    pub fn open (path: &str) -> Result<Self> {
        Ok(Self {
            writer: Writer::from_writer(
                OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(path)?
            )
        })
    }

    /// Impure (writes to file)
    /// # Panics
    /// Panics if passed a task with a `None` `id`
    ///
    /// # Errors
    /// Returns an error if the time cannot be written to file
    pub fn log_time (&mut self, time: Duration, task: &Task) -> Result<()> {
        Ok(self.writer.write_record([
            &task.name,
            &task.id.expect("The task should have an id").to_string(),
            &time.to_string(),
            &today_string(),
            &now_string(),
        ])?)
    }
}
