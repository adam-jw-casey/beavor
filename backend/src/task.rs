use std::cmp::max;

use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use chrono::{
    NaiveDate,
    Duration,
};

use crate::utils::{today_date,parse_date};
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

impl TryFrom<SqliteRow> for Task {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Task {
            category:                         row.get::<String, &str>("Category"),
            finished:                         row.get::<bool,   &str>("Finished"),
            name:                             row.get::<String, &str>("Name"),
            _time_budgeted: Duration::minutes(row.get::<i64,    &str>("Budget")),
            time_needed:    Duration::minutes(row.get::<i64,    &str>("Time")),
            time_used:      Duration::minutes(row.get::<i64,    &str>("Used")),
            next_action_date:     parse_date(&row.get::<String, &str>("NextAction"))?,
            due_date:                         row.get::<String, &str>("DueDate").try_into()?,
            notes:                            row.get::<String, &str>("Notes"),
            id:                               row.get::<Option<u32>, &str>("TaskID"),
            date_added:           parse_date(&row.get::<String, &str>("DateAdded"))?,
            links:                            Vec::new(),
        })
    }
}

/// Stores the data for a hyperlink. This is a thin data class
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Default)]
pub struct Hyperlink {
    pub url:     String,
    pub display: String,
    pub id:      usize,
}

impl TryFrom<SqliteRow> for Hyperlink {
    type Error = std::convert::Infallible;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Hyperlink {
            url:     row.get::<String, &str>("Url"),
            display: row.get::<String, &str>("Display"),
            id:      row.get::<u32, &str>("rowid") as usize,
        })
    }
}
