use pyo3::prelude::{
    pyclass,
    pymethods,
};

use chrono::{
    NaiveDate,
    Datelike,
    Weekday
};

use crate::{
    today_date,
    DueDate,
    Task
};

use std::collections::HashMap;

fn num_week_days_from(d1: NaiveDate, d2: NaiveDate) -> u32{
    let weeks_between = (d2-d1).num_weeks() as u32;

    let marginal_weekdays: u32 = match d2.weekday(){
        Weekday::Sat | Weekday::Sun => match d1.weekday(){
            Weekday::Sat | Weekday::Sun => 0,
            weekday1 => Weekday::Fri.number_from_monday() - weekday1.number_from_monday() + 1,
        },
        weekday2 => match d1.weekday(){
            Weekday::Sat | Weekday::Sun => weekday2.number_from_monday() - Weekday::Mon.number_from_monday(),
            weekday1 => (weekday2.number_from_monday() - weekday1.number_from_monday()).rem_euclid(5) + 1,
        },
    };

    weeks_between * 5 + marginal_weekdays
}

#[pyclass]
pub struct Calendar{
    #[pyo3(get)]
    days_off: Vec<NaiveDate>,
    #[pyo3(get)]
    workloads: HashMap<NaiveDate, u32>,
}

impl Calendar{
    /// Counts and returns the number of non-weekend days off between d1 and d2
    fn num_days_off_from(&self, d1: NaiveDate, d2: NaiveDate) -> u32 {
        self.days_off
            .iter()
            .filter(|d: &&NaiveDate| d1 <= **d && **d <= d2)
            .count().try_into().expect("This should fit in a u32")
    }

    /// Counts and returns the number of working days between d1 and d2
    /// NOTE: this will be incorrect if a weekend day has been marked as a day off
    fn num_work_days_from(&self, d1: NaiveDate, d2: NaiveDate) -> u32{
        num_week_days_from(d1, d2) - self.num_days_off_from(d1, d2)
    }

    /// Calculates and records the number of minutes that need to be worked each day
    fn calculate_workloads (&mut self, tasks: Vec<Task>){
        // Update self.workloads
        todo!()
    }

    /// Returns a boolean representing whether a given date is a work day
    fn is_work_day(&self, date: NaiveDate) -> bool{
        !self.days_off.contains(&date) && !vec![Weekday::Sun, Weekday::Sat].contains(&date.weekday())
    }
}

#[pymethods]
impl Calendar{
    #[new]
    fn __new__(days_off: Vec<NaiveDate>, tasks: Vec<Task>) -> Self{
        let mut calendar = Calendar{
            days_off,
            workloads: HashMap::<NaiveDate, u32>::new(),
        };

        calendar.calculate_workloads(tasks);

        calendar
    }

    /// Returns a boolean representing whether a given task can be worked on on a given date
    fn is_available_on_day(&self, task: Task, date: NaiveDate) -> bool{
        let before_end: bool = match task.due_date{
            DueDate::NONE => true,
            DueDate::Date(raw_due_date) => date <= raw_due_date || date == today_date(),
            DueDate::ASAP => date == today_date(),
        };

        let after_beginning: bool = task.next_action_date <= date;
        
        before_end && after_beginning && self.is_work_day(date)
    }

    /// Returns the number of minutes of work that need to be done on a given date
    fn workload_on_day(&self, date: NaiveDate) -> u32{
        self.workloads[&date]
    }
}

#[cfg(test)]
#[allow(deprecated)]
#[allow(clippy::zero_prefixed_literal)]
mod tests{
    use super::*;

    #[test]
    fn test_week_days_from() {
        assert_eq!(
            num_week_days_from(
                NaiveDate::from_ymd(2023, 08, 21),
                NaiveDate::from_ymd(2023, 08, 25)
            ),
            5 // This is a simple workweek
        );

        assert_eq!(
            num_week_days_from(
                NaiveDate::from_ymd(2023, 08, 11),
                NaiveDate::from_ymd(2023, 08, 14)
            ),
            2 // Friday to Monday
        );

        assert_eq!(
            num_week_days_from(
                NaiveDate::from_ymd(2023, 08, 1),
                NaiveDate::from_ymd(2023, 08, 23)
            ),
        17 // Multiple weeks, starting day of week is earlier
        );

        assert_eq!(
            num_week_days_from(
                NaiveDate::from_ymd(2023, 08, 4),
                NaiveDate::from_ymd(2023, 08, 23)
            ),
            14 // Multiple weeks, starting day of week is later
        );

        assert_eq!(
            num_week_days_from(
                NaiveDate::from_ymd(2023, 08, 19),
                NaiveDate::from_ymd(2023, 08, 27)
            ),
            5 // Start and end on a weekend
        );
    }
}