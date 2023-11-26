use std::cmp::max;
use std::ops::Sub;
use std::num::IntErrorKind;

use serde::{Deserialize, Serialize};

use chrono::{
    NaiveDate,
    Datelike,
    Weekday,
    Duration,
};

use crate::{
    Task,
    DueDate,
    utils::today_date,
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

#[derive(Clone, Default, Debug)]
pub struct Schedule{
    days_off: Vec<NaiveDate>,
    workloads: HashMap<NaiveDate, Duration>,
    work_week: WorkWeek,
}

impl Schedule{
    #[must_use] pub fn new(days_off: Vec<NaiveDate>, tasks: Vec<Task>, work_week: WorkWeek) -> Self{
        let mut schedule = Schedule{
            days_off,
            workloads: HashMap::<NaiveDate, Duration>::new(),
            work_week,
        };

        schedule.calculate_workloads(tasks);

        schedule
    }

    /// Calculates and returns the number of minutes per day you would have to work on the task to
    /// complete it between the day it is available and the day it is due, if there is a due date
    fn workload_per_day(&self, task: &Task) -> Option<Duration>{
        Some(task.time_remaining() / max(1, self.num_days_to_work_on(task)?.try_into().expect("This should be few enough days to fit in an i32")))
    }

    /// Returns an iterator over the working days between two dates, including both ends
    fn work_days_from(&self, d1: NaiveDate, d2: NaiveDate) -> impl Iterator<Item = NaiveDate> + '_{
        DateIterator::new(d1, Some(d2))
            .filter(|d| self.is_work_day(*d))
    }

    /// Returns an iterator over the days a task can be worked on, or nothing if the task has no due
    /// date (i.e., the range is undefined)
    fn work_days_for_task(&self, task: &Task) -> Option<Box<impl Iterator<Item = NaiveDate> + '_>> {
        self.last_available_date_for_task(task)
            .map(|due_date| Box::new(self.work_days_from(self.first_available_date_for_task(task), due_date)))
    }

    /// Returns the number of days a task can be worked on, if there is a due date
    fn num_days_to_work_on(&self, task: &Task) -> Option<u32> {
        Some(self.work_days_for_task(task)?.count().try_into().expect("This fails on huge numbers"))
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
    #[must_use] pub fn first_available_date_for_task(&self, task: &Task) -> NaiveDate{
        max(task.next_action_date, self.next_work_day())
    }

    /// Returns the last date that a task can be worked on
    #[must_use] pub fn last_available_date_for_task(&self, task: &Task) -> Option<NaiveDate>{
        match task.due_date{
            DueDate::Never => None,
            DueDate::Date(due_date) => Some(max(due_date, self.next_work_day())),
            DueDate::Asap => Some(self.first_available_date_for_task(task)),
        }
    }

    /// Calculates and records the number of minutes that need to be worked each day
    fn calculate_workloads (&mut self, tasks: Vec<Task>){
        // Cannot be done on self.workloads in-place due to borrow rules with the filter in the for-loop below
        let mut workloads = HashMap::<NaiveDate, Duration>::new();

        for task in tasks{
            if let Some(workload_per_day) = self.workload_per_day(&task){
                for day in self.work_days_for_task(&task).expect("We've already checked that workload_per_day is not None, so this will not be None"){
                    workloads
                        .entry(day)
                        .and_modify(|workload| *workload = *workload + workload_per_day)
                        .or_insert(workload_per_day);
                }
            };
        }

        self.workloads = workloads;
    }

    /// Returns a boolean representing whether a given task can be worked on on a given date
    #[must_use] pub fn is_available_on_day(&self, task: &Task, date: NaiveDate) -> bool{
        let before_end = self.last_available_date_for_task(task).map_or(true, |available_date| date <= available_date);

        let after_beginning = task.next_action_date <= date;

        before_end && after_beginning
    }

    /// Returns the number of minutes of work that need to be done on a given date
    #[must_use] pub fn workload_on_day(&self, date: NaiveDate) -> Option<Duration>{
        if self.is_work_day(date) && date >= today_date(){
            Some(*self.workloads.get(&date)
                .unwrap_or(&Duration::minutes(0)))
        }else{None}
    }

    /// Returns a boolean representing whether a given date is a work day
    #[must_use] pub fn is_work_day(&self, date: NaiveDate) -> bool{
        !self.days_off.contains(&date) && self.work_week.workdays().contains(&date.weekday())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkWeek{
    // TODO It would be interesting to assign different time allocations to different task categories
    days: HashMap<Weekday, WorkDay>
}

impl WorkWeek{
    pub fn workdays(&self) -> Vec<Weekday>{
        self.days.iter()
            .filter(|(_, workday)| workday.hours_this_day() > 0.try_into().unwrap())
            .map(|(weekday, _)| *weekday)
            .collect()
    }
}

impl Default for WorkWeek{
    fn default() -> Self {
        let mut days = HashMap::new();

        let full_day = HourRange{
            start_hour: 8.try_into().unwrap(),
            end_hour: 17.try_into().unwrap(),
        };

        days.insert(Weekday::Mon, WorkDay::new(Some(full_day)));
        days.insert(Weekday::Tue, WorkDay::new(Some(full_day)));
        days.insert(Weekday::Wed, WorkDay::new(Some(full_day)));
        days.insert(Weekday::Thu, WorkDay::new(Some(full_day)));
        days.insert(Weekday::Fri, WorkDay::new(Some(full_day)));
        days.insert(Weekday::Sat, WorkDay::new(None));
        days.insert(Weekday::Sun, WorkDay::new(None));

        WorkWeek{days}
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WorkDay{
    hours_of_work: Option<HourRange>,
}

impl WorkDay{
    #[must_use] pub fn new(hours: Option<HourRange>) -> Self{
        Self{
            hours_of_work: hours,
        }
    }

    pub fn hours_this_day(&self) -> DayHour{
        match self.hours_of_work{
            Some(hours) => (hours.end_hour - hours.start_hour).expect("Start should be earlier than end"),
            None => 0.try_into().unwrap(),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
// TODO should not allow end earlier than beginning
pub struct HourRange{
    start_hour: DayHour,
    end_hour:   DayHour,
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct DayHour{
    value: u8
}

impl Sub for DayHour{
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.value < rhs.value{
            None
        }else{
            Some((self.value - rhs.value).try_into().unwrap())
        }
    }
}

impl TryFrom<u8> for DayHour{
    type Error = IntErrorKind;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 24{
            Err(IntErrorKind::PosOverflow)
        }else{
            Ok(DayHour{value})
        }
    }
}

impl From<DayHour> for u8{
    fn from(val: DayHour) -> Self {
        val.value
    }
}

#[allow(clippy::zero_prefixed_literal)]
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    // TODO These all use the year 3000 as being arbitrarily far in the future. This is probably
    // fine, but is still ugly
    fn test_schedule(){
        let task = Task{
            next_action_date: NaiveDate::from_ymd_opt(3000, 01, 01).unwrap(),
            due_date: DueDate::Date(NaiveDate::from_ymd_opt(3000, 01, 03).unwrap()),
            time_needed: Duration::minutes(60),
            ..Default::default()
        };

        let schedule = Schedule::new(vec![NaiveDate::from_ymd_opt(3000,01,08).unwrap()], vec![task.clone()]);

        assert!(schedule.is_available_on_day(&task, NaiveDate::from_ymd_opt(3000,01,01).unwrap()));
        assert!(schedule.is_available_on_day(&task, NaiveDate::from_ymd_opt(3000,01,02).unwrap()));
        assert!(schedule.is_available_on_day(&task, NaiveDate::from_ymd_opt(3000,01,03).unwrap()));
        assert!(!schedule.is_available_on_day(&task, NaiveDate::from_ymd_opt(3000,01,04).unwrap()));

        assert!(schedule.is_work_day(NaiveDate::from_ymd_opt(3000,01,06).unwrap()));
        assert!(!schedule.is_work_day(NaiveDate::from_ymd_opt(3000,01,05).unwrap()));
        assert!(!schedule.is_work_day(NaiveDate::from_ymd_opt(3000,01,08).unwrap()));

        assert_eq!(schedule.workload_on_day(NaiveDate::from_ymd_opt(3000,01,02).unwrap()), Some(Duration::minutes(20)));
    }
}
