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
            date_added:         today_date(),
            finished:           false,
            name:               String::new(),
            _time_budgeted:     Duration::zero(),
            time_needed:        Duration::zero(),
            time_used:          Duration::zero(),
            notes:              String::new(),
            id:                 None,
            links:              Vec::new(),
        }
    }
}

impl TryFrom<SqliteRow> for BoundedTask {
    type Error = anyhow::Error;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(
            Self{
                start:                                row.get::<&str, &str>("NextAction").try_into()?,
                end:                                  row.get::<&str, &str>("DueDate").try_into()?,
                task: Task {
                    category:                         row.get::<String, &str>("Category"),
                    finished:                         row.get::<bool,   &str>("Finished"),
                    name:                             row.get::<String, &str>("Name"),
                    _time_budgeted: Duration::minutes(row.get::<i64,    &str>("Budget")),
                    time_needed:    Duration::minutes(row.get::<i64,    &str>("Time")),
                    time_used:      Duration::minutes(row.get::<i64,    &str>("Used")),
                    notes:                            row.get::<String, &str>("Notes"),
                    id:                               row.get::<Option<u32>, &str>("TaskID"),
                    date_added:           parse_date(&row.get::<String, &str>("DateAdded"))?,
                    links:                            Vec::new(),
                }
            }
        )
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
            id:      row.get::<u32,    &str>("rowid") as usize,
        })
    }
}

// Ensure there aren't public outside backend
pub struct BoundedTask {
    pub task:  Task,
    pub start: DBStart,
    pub end:   DBEnd,
}

pub enum DBStart {
    Raw(NaiveDate),
    Milestone(Id),
}

pub enum DBEnd {
    Raw(DueDate),
    Milestone(Id),
}

impl TryFrom<&str> for DBStart {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.parse::<u32>() {
            Ok(id) => Ok(Self::Milestone(Some(id))),
            Err(_) => match parse_date(value) {
                Ok(date) => Ok(Self::Raw(date)),
                Err(e) => Err(e),
            },
        }
    }
}

impl TryFrom<&str> for DBEnd {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.parse::<u32>() {
            Ok(id) => Ok(Self::Milestone(Some(id))),
            Err(_) => match DueDate::try_from(value) {
                Ok(due_date) => Ok(Self::Raw(due_date)),
                Err(e) => Err(e),
            },
        }
    }
}
