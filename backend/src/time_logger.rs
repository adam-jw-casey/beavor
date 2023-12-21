use std::fs::File;
use std::path::Path;
use std::io::{Error, ErrorKind,};

use chrono::Duration;
use anyhow::Result;

use csv::Writer;

use crate::Task;

struct TimeLogger {
    writer: Writer<File>,
}

impl TimeLogger {
    pub fn new_logfile(path: &str) -> Result<Self> {
        if Path::exists(Path::new(path)) {
            return Err(Error::new(ErrorKind::AlreadyExists, format!("Cannot create a logfile at: {path}\nFile exists!")).into());
        }

        let mut logger = Self::open(path)?;

        logger.writer.write_record(["Task", "TaskID", "Time Worked"])?;

        Ok(logger)
    }

    pub fn open (path: &str) -> Result<Self> {
        Ok(Self {
            writer: Writer::from_path(path)?
        })
    }

    pub fn log_time (&mut self, time: Duration, task: &Task) -> Result<()>{
        Ok(self.writer.write_record([
            &task.name,
            &task.id.expect("The task should have an id").to_string(),
            &time.to_string()]
        )?)
    }
}
