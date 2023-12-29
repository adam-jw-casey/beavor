use std::cmp::max;

use chrono::{
    NaiveDate,
    Duration,
};

use crate::utils::today_date;
use crate::due_date::DueDate;

pub type Id = Option<u32>;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
pub struct Task {
    pub category:         String,
    pub finished:         bool,
    pub name:             String,
    pub _time_budgeted:   Duration,
    pub time_needed:      Duration,
    pub time_used:        Duration,
    pub notes:            String,
    pub date_added:       NaiveDate,
    pub next_action_date: NaiveDate,
    pub due_date:         DueDate,
    pub id:               Id,
    pub links:            Vec<Hyperlink>,
}

impl Task {
    /// Equivalent to `Task::default()`
    /// # Examples
    /// ```
    /// use backend::Task;
    ///
    /// assert_eq!(Task::default(), Task::new());
    /// ```
    #[must_use] pub fn new() -> Self {
        Self::default()
    }

    /// Returns the amount of time that still needs to be worked on the task
    #[must_use] pub fn time_remaining(&self) -> Duration {
        max(self.time_needed - self.time_used, Duration::zero())
    }
}

impl std::default::Default for Task {
    fn default() -> Self {
        Task {
            category:           "Work".into(),
            next_action_date:   today_date(),
            date_added:         today_date(),
            finished:           false,
            name:               String::new(),
            _time_budgeted:     Duration::zero(),
            time_needed:        Duration::zero(),
            time_used:          Duration::zero(),
            notes:              String::new(),
            due_date:           DueDate::Asap,
            id:                 None,
            links:              Vec::new(),
        }
    }
}

/// Stores the data for a hyperlink. This is a thin data class
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Default)]
pub struct Hyperlink {
    pub url:     String,
    pub display: String,
    pub id:      usize,
}
