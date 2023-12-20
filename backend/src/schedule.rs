use std::cmp::{max, min};

use serde::{Deserialize, Serialize};

use chrono::{
    NaiveDate,
    Datelike,
    Weekday,
    Duration,
    NaiveTime,
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

pub type WorkLoads = HashMap<NaiveDate, WorkDay>;

#[derive(Clone, Debug)]
pub struct WorkDay  {
    working_hours: WorkingHours,
    time_per_task: TimePerTask,
}

impl WorkDay {
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
    #[must_use] pub fn new (days_off: Vec<NaiveDate>, tasks: &Vec<Task>, work_week: WorkWeek) -> Self{
        let mut schedule = Schedule{
            days_off,
            workloads: WorkLoads::new(),
            work_week,
        };

        schedule.calculate_workloads(tasks);

        schedule
    }

    // These warnings occur because of the `as` below, but this operation is actually infallible
    // due to the known range for the values involved
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    #[must_use] pub fn time_remaining_today(&self) -> Option<Duration> {
        self.work_week.days[&today_date().weekday()]
            .hours_of_work
            .map(|hour_range| hour_range.end - max(now_time(), hour_range.start))
    }

    // TODO methods like these should absolutely be on WorkDay
    #[must_use] pub fn time_available_on_date(&self, date: NaiveDate) -> Option<Duration> {
        if self.is_work_day(date) {
            Some(self.work_week.working_hours_on_day(date).working_time() - self.get_workload_on_day(date).unwrap_or(Duration::hours(0)))
        } else {
            None
        }
    }

    /// Returns an iterator over the working days between two dates, including both ends
    fn work_days_from(&self, d1: NaiveDate, d2: NaiveDate) -> impl Iterator<Item = NaiveDate> + '_ {
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
        self.frontload_workloads(tasks);
    }

    /// One variant of the workload calculation
    /// This sorts the tasks from first due to last, and schedules work as early as possible
    fn frontload_workloads (&mut self, tasks: &Vec<Task>){
        // Cannot be done on self.workloads in-place due to borrow rules with the filter in the for-loop below
        let mut workloads = WorkLoads::new();

        // Sort from first to last due
        let mut sorted_tasks = tasks.clone();
        sorted_tasks.sort_unstable_by(|t1, t2| t1.due_date.cmp(&t2.due_date));

        for task in tasks {
            // Track the time that has not yet been assigned to a day
            let mut time_to_assign = task.time_remaining();

            if let Some(task_days) = self.work_days_for_task(task) {
                for day in task_days {
                    if time_to_assign.num_seconds() == 0 {
                        break; // Don't continue looping once all time is assigned
                    }
                    
                    // Find how much time can be allocated to this day from this task
                    let workload_for_day = min(self.time_available_on_date(day).expect("This will be some because all work days have non-None time"), time_to_assign);
                    // Remove the time to be allocated from the remaining time for the task
                    time_to_assign = time_to_assign - workload_for_day;

                    workloads
                        .entry(day)
                        .or_insert(WorkDay::new(WorkingHours::new(self.work_week.working_hours_on_day(day).hours_of_work)))
                        .add(task, workload_for_day);
                }

                // If time remains, assign to final day
                if time_to_assign.num_seconds() != 0 {
                    workloads.get_mut(
                            &self.work_days_for_task(task)
                                .expect("We already checked this is not None")
                                .last()
                                .expect("work_days_for_task never returns an empty iterator")
                        )
                        .expect("This will not be None because we've already inserted a value in the previous loop")
                        .add(task, time_to_assign);
                }
            } else {
                todo!("TBD how to handle tasks that are not available for any days")
            }
        }

        self.workloads = workloads;
    }

    /// Returns a boolean representing whether a given task can be worked on on a given date
    #[must_use] pub fn is_available_on_day(&self, task: &Task, date: NaiveDate) -> bool {
        let before_end = self.last_available_date_for_task(task).map_or(true, |available_date| date <= available_date);

        let after_beginning = task.next_action_date <= date;

        before_end && after_beginning
    }

    // TODO this should return an empty hashmap, not None, if there are no tasks on that day. It might be be that nothing was scheduled.
    /// Returns the duration of work that need to be done on a given date
    #[must_use] pub fn get_time_per_task_on_day(&self, date: NaiveDate) -> Option<&TimePerTask> {
        Some(
            &self.workloads
                .get(&date)? // The duration of each task assigned to the day
                .time_per_task
        )
    }

    // TODO very inconsistent use of word "workload" to refer to the WorkLoad type vs. a duration
    // TODO this should be a thin wrapper around a method on WorkDay
    #[must_use] pub fn get_workload_on_day(&self, date: NaiveDate) -> Option<Duration> {
        if self.is_work_day(date) {
            Some(self.get_time_per_task_on_day(date)
                .unwrap_or(&HashMap::new())
                .iter()
                .map(|(_id, d)| *d)
                .sum())
        } else {
            None
        }
    }

    /// Returns a boolean representing whether a given date is a work day
    #[must_use] pub fn is_work_day(&self, date: NaiveDate) -> bool {
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
            .filter(|(_, workday)| workday.working_time() > Duration::hours(0))
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

        #[allow(deprecated)]
        let full_day = HourRange::new(NaiveTime::from_hms(8,0,0), NaiveTime::from_hms(17,0,0)).expect("This is const and will never fail");

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

    #[must_use] pub fn working_time(&self) -> Duration{
        match self.hours_of_work{
            Some(hours) => hours.duration(),
            None => Duration::hours(0),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct HourRange{
    start: NaiveTime,
    end:   NaiveTime,
}

impl HourRange {
    #[must_use] pub fn new(start: NaiveTime, end: NaiveTime) -> Option<Self> {
        if end > start {
            Some(Self{start, end})
        } else {
            None
        }
    }

    #[must_use] pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}
