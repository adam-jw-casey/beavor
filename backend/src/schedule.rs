use std::cmp::max;
use std::ops::Sub;
use std::num::IntErrorKind;

use serde::{Deserialize, Serialize};

use chrono::{
    NaiveDate,
    Datelike,
    Weekday,
    Duration,
    Timelike,
};

use crate::{
    Task,
    task::Id,
    DueDate,
    utils::{
        today_date,
        now_time,
    },
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

pub type WorkLoads = HashMap<NaiveDate, WorkingDay>;

#[derive(Clone, Debug)]
struct WorkingDay  {
    working_hours: WorkingHours,
    time_per_task: TimePerTask,
}

impl WorkingDay {
    fn add (&mut self, task: &Task, duration: Duration) {
        let current_duration = self.time_per_task.entry(task.id).or_insert(Duration::minutes(0));

        *current_duration = *current_duration + duration;
    }

    fn new (working_hours: WorkingHours) -> Self {
        Self {
            working_hours,
            time_per_task: TimePerTask::default(),
        }
    }
}

pub type TimePerTask = HashMap<Id, Duration>;

#[derive(Clone, Default, Debug)]
pub struct Schedule  {
    days_off: Vec<NaiveDate>,
    workloads: WorkLoads,
    work_week: WorkWeek,
}

impl Schedule {
    // These warnings occur because of the `as` below, but this operation is actually infallible
    // due to the known range for the values involved
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    #[must_use] pub fn hours_remaining_today(&self) -> Option<i8> {
        self.work_week.days[&today_date().weekday()]
            .hours_of_work
            .map(|hour_range| now_time().hour() as i8 - hour_range.end_hour.value as i8)
    }

    #[must_use] pub fn new (days_off: Vec<NaiveDate>, tasks: &Vec<Task>, work_week: WorkWeek) -> Self{
        let mut schedule = Schedule{
            days_off,
            workloads: WorkLoads::new(),
            work_week,
        };

        schedule.calculate_workloads(tasks);

        schedule
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
    fn calculate_workloads (&mut self, tasks: &Vec<Task>){
        // Cannot be done on self.workloads in-place due to borrow rules with the filter in the for-loop below
        let mut workloads = WorkLoads::new();

        for task in tasks{
            for day in self.work_days_for_task(task).expect("We've already checked that workload_per_day is not None, so this will not be None"){
                
                let workload_on_day = self.calculate_workload_on_day(); // TODO new calculations for workload on a day for a task

                workloads
                    .entry(day)
                    .or_insert(WorkingDay::new(WorkingHours::new(self.work_week.working_hours_on_day(day).hours_of_work)))
                    .add(task, workload_on_day);
            }
        }

        self.workloads = workloads;
    }

    #[must_use] fn calculate_workload_on_day (&self) -> Duration {
        todo!()
    }

    /// Returns a boolean representing whether a given task can be worked on on a given date
    #[must_use] pub fn is_available_on_day(&self, task: &Task, date: NaiveDate) -> bool{
        let before_end = self.last_available_date_for_task(task).map_or(true, |available_date| date <= available_date);

        let after_beginning = task.next_action_date <= date;

        before_end && after_beginning
    }

    /// Returns the duration of work that need to be done on a given date
    #[must_use] pub fn get_workload_on_day(&self, date: NaiveDate) -> Option<&TimePerTask>{
        Some(&self.workloads
            .get(&date)? // The duration of that task assigned to the day
            .time_per_task)
    }

    /// Returns a boolean representing whether a given date is a work day
    #[must_use] pub fn is_work_day(&self, date: NaiveDate) -> bool{
        !self.days_off.contains(&date) && self.work_week.workdays().contains(&date.weekday())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkWeek{
    days: HashMap<Weekday, WorkingHours>
}

impl WorkWeek{
    // The `try_into` is know to be infallible because 0 < 24
    #[allow(clippy::missing_panics_doc)]
    #[must_use] pub fn workdays(&self) -> Vec<Weekday>{
        self.days.iter()
            .filter(|(_, workday)| workday.num_working_hours_this_day() > 0.try_into().unwrap())
            .map(|(weekday, _)| *weekday)
            .collect()
    }

    #[must_use] pub fn working_hours_on_day(&self, day: NaiveDate) -> WorkingHours {
        self.days[&day.weekday()]
    }
}

impl Default for WorkWeek{
    fn default() -> Self {
        let mut days = HashMap::new();

        let full_day = HourRange{
            start_hour: 8.try_into().unwrap(),
            end_hour: 17.try_into().unwrap(),
        };

        days.insert(Weekday::Mon, WorkingHours::new(Some(full_day)));
        days.insert(Weekday::Tue, WorkingHours::new(Some(full_day)));
        days.insert(Weekday::Wed, WorkingHours::new(Some(full_day)));
        days.insert(Weekday::Thu, WorkingHours::new(Some(full_day)));
        days.insert(Weekday::Fri, WorkingHours::new(Some(full_day)));
        days.insert(Weekday::Sat, WorkingHours::new(None));
        days.insert(Weekday::Sun, WorkingHours::new(None)); // There has got to be a better way of inserting all these? From an array?

        WorkWeek{days}
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WorkingHours{
    hours_of_work: Option<HourRange>,
}

impl WorkingHours{
    #[must_use] pub fn new(hours: Option<HourRange>) -> Self{
        Self{
            hours_of_work: hours,
        }
    }

    #[must_use] pub fn num_working_hours_this_day(&self) -> DayHour{
        match self.hours_of_work{
            Some(hours) => (hours.end_hour - hours.start_hour).expect("Start should be earlier than end"),
            None => 0.try_into().unwrap(),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct HourRange{
    start_hour: DayHour,
    end_hour:   DayHour,
}

// TODO this might be more useful as a Duration type than an integer type.
// This is an integer type bounded from 0 to 24
// It it used to to reprent a time of day (24-hour clock) but also a number of hours over the course of a day
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
        // Unsigned so don't need to check lower bound
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
