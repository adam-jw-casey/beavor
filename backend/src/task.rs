use chrono::NaiveDate;

use crate::utils::today_date;
use crate::due_date::DueDate;

pub type Id = Option<u32>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
pub struct Task{
    pub category:         String,
    pub finished:         bool,
    pub name:             String,
    pub _time_budgeted:   u32,
    pub time_needed:      u32,
    pub time_used:        u32,
    pub notes:            String,
    pub date_added:       NaiveDate,
    pub next_action_date: NaiveDate,
    pub due_date:         DueDate,
    pub id:               Id,
}

impl Task{
    #[must_use] pub fn new() -> Self{
        Self::default()
    }

    #[must_use] pub fn time_remaining(&self) -> u32{
        self.time_needed - self.time_used
    }
}

impl std::default::Default for Task{
    #[allow(unconditional_recursion)]
    fn default() -> Self{
        Task{
            category:         "Work".into(),
            next_action_date: today_date(),
            date_added:       today_date(),
            ..Default::default()
        }
    }
}
