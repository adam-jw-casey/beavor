use std::cmp::max;

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
    Task,
    DueDate,
    today_date,
};

use std::collections::HashMap;

/// This stores more state than necessary but I don't feel like optimizing it and I doubt it'll be
/// a bottleneck anytime soon
pub struct DateIterator{
    prev: NaiveDate,
    next: NaiveDate,
    last: Option<NaiveDate>,
}

impl DateIterator{
    fn new(start: NaiveDate, end: Option<NaiveDate>) -> Self{
        DateIterator{
            prev: start,
            next: start,
            last: end,
        }
    }
}

impl Iterator for DateIterator{
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.last{
            Some(date) => {
                self.prev = self.next;
                self.next = self.prev.succ_opt().expect("This panics on huge dates");
                Some(self.prev).filter(|d| *d <= date)
            },
            None => None,
        }
    }
}

#[pyclass]
pub struct Schedule{
    #[pyo3(get)]
    days_off: Vec<NaiveDate>,
    #[pyo3(get)]
    workloads: HashMap<NaiveDate, u32>,
}

impl Schedule{
    pub fn new(days_off: Vec<NaiveDate>, tasks: Vec<Task>) -> Self{
        let mut schedule = Schedule{
            days_off,
            workloads: HashMap::<NaiveDate, u32>::new(),
        };

        schedule.calculate_workloads(tasks);

        schedule
    }

    // TODO - This does not consider the amount of time already worked today, or the time of day,
    //        so it continues to assign the same amount of time to the current day, regardless of
    //        how much of the day is left.
    //        Consider adjusting  `num_days_to_work_on` to incorporate the current time-of-day when
    //        the date range overlaps today
    /// Calculates and returns the number of minutes per day you would have to work on the task to
    /// complete it between the day it is available and the day it is due
    fn workload_per_day(&self, task: &Task) -> Option<u32>{
        Some(task.time_remaining() / max(1, self.num_days_to_work_on(task)))
    }

    /// Returns an iterator over the working days between two dates, including both ends
    fn work_days_from(&self, d1: NaiveDate, d2: NaiveDate) -> impl Iterator<Item = NaiveDate> + '_{
        DateIterator::new(d1, Some(d2))
            .filter(|d| self.is_work_day(*d))
    }

    /// Returns an iterator over the days a task can be worked on, or nothing if the task has no due
    /// date (i.e., the range is undefined)
    // TODO -> this is begging for an enum. The empty iterator is being abused here, and this will
    // be confusing down the line
    fn work_days_for_task(&self, task: &Task) -> Box<dyn Iterator<Item = NaiveDate> + '_> {
        match self.last_available_date_for_task(task){
            Some(due_date) => Box::new(self.work_days_from(
                self.first_available_date_for_task(task),
                due_date,
            )),
            None => Box::new(std::iter::empty::<NaiveDate>()),
        }

    }

    /// Returns the number of days a task can be worked on, or 0 if the task has no due date
    // TODO see comment above re: enum
    fn num_days_to_work_on(&self, task: &Task) -> u32 {
        self.work_days_for_task(task).count().try_into().expect("This fails on huge numbers")
    }

    /// Returns the date of the soonest work day, including today
    fn next_work_day(&self) -> NaiveDate {
        let mut day = today_date();
        while !self.is_work_day(day){
            day = day.succ_opt().expect("This will fail on huge dates");
        }

        day
    }

    /// Returns first date that a task can be worked on
    pub fn first_available_date_for_task(&self, task: &Task) -> NaiveDate{
        max(task.next_action_date, self.next_work_day())
    }

    /// Returns the last date that a task can be worked on
    pub fn last_available_date_for_task(&self, task: &Task) -> Option<NaiveDate>{
        match task.due_date{
            DueDate::NONE => None,
            DueDate::Date(due_date) => Some(max(due_date, self.next_work_day())),
            DueDate::ASAP => Some(self.first_available_date_for_task(task)),
        }
    }
}

#[pymethods]
impl Schedule{
    /// Returns a boolean representing whether a given task can be worked on on a given date
    fn is_available_on_day(&self, task: Task, date: NaiveDate) -> bool{
        let before_end = self.last_available_date_for_task(&task).map(|available_date| date <= available_date).unwrap_or(true);

        let after_beginning = task.next_action_date <= date;

        before_end && after_beginning
    }

    /// Returns the number of minutes of work that need to be done on a given date
    fn workload_on_day(&self, date: NaiveDate) -> u32{
        *self.workloads.get(&date)
            .unwrap_or(&0)
    }

    /// Calculates and records the number of minutes that need to be worked each day
    fn calculate_workloads (&mut self, tasks: Vec<Task>){
        // Cannot be done on self.workloads in-place due to borrow rules with the filter in the for-loop below
        let mut workloads = HashMap::<NaiveDate, u32>::new();

        for task in tasks{
            if let Some(workload_per_day) = self.workload_per_day(&task){
                for day in self.work_days_for_task(&task){
                    workloads
                        .entry(day)
                        .and_modify(|workload| *workload += workload_per_day)
                        .or_insert(workload_per_day);
                }
            }else{continue};
        }

        self.workloads = workloads;
    }

    /// Returns a boolean representing whether a given date is a work day
    fn is_work_day(&self, date: NaiveDate) -> bool{
        !self.days_off.contains(&date) && ![Weekday::Sun, Weekday::Sat].contains(&date.weekday())
    }
}
