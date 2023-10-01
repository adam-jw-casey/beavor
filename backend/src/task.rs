use chrono::NaiveDate;

use crate::utils::today_date;

use crate::due_date::DueDate;

pub type Id = Option<u32>;

#[derive(Clone, Debug)]
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
    pub fn time_remaining(&self) -> u32{
        self.time_needed - self.time_used
    }
}


impl std::default::Default for Task{
    fn default() -> Self{
        Task{
            category:         "Work".into(),
            finished:         false,
            name:             "".into(),
            _time_budgeted:   0,
            time_needed:      0,
            time_used:        0,
            next_action_date: today_date(),
            due_date:         DueDate::Date(today_date()),
            notes:            "".into(),
            id:               None,
            date_added:       today_date(),
        }
    }
}
