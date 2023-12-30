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

// TODO This stores more state than necessary but I don't feel like optimizing it and I doubt it'll be a bottleneck anytime soon
pub struct DateIterator {
    prev: NaiveDate,
    next: NaiveDate,
    last: Option<NaiveDate>,
}

impl DateIterator {
    /// Creates an iterator that will return every date, in order, from start to end, inclusive
    /// # Examples
    /// ```
    /// use backend::schedule::DateIterator;
    /// use chrono::NaiveDate;
    ///
    /// let it = DateIterator::new(NaiveDate::from_ymd(2024,01,01), Some(NaiveDate::from_ymd(2024,01,05)));
    ///
    /// let dates: Vec<NaiveDate> = it.collect();
    ///
    /// assert_eq!(
    ///     dates,
    ///     vec![
    ///         NaiveDate::from_ymd(2024,01,01),
    ///         NaiveDate::from_ymd(2024,01,02),
    ///         NaiveDate::from_ymd(2024,01,03),
    ///         NaiveDate::from_ymd(2024,01,04),
    ///         NaiveDate::from_ymd(2024,01,05)
    ///     ]
    /// )
    /// ```
    #[must_use] pub fn new(start: NaiveDate, end: Option<NaiveDate>) -> Self {
        DateIterator {
            prev: start,
            next: start,
            last: end,
        }
    }
}

impl Iterator for DateIterator {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.last {
            Some(date) => {
                self.prev = self.next;
                self.next = self.prev.succ_opt().expect("This panics on huge dates");
                Some(self.prev).filter(|d| *d <= date)
            },
            None => None,
        }
    }
}

pub type WorkDays = HashMap<NaiveDate, WorkDay>;

/// This maps task ids to a duration of time.
/// This type is intended to represent a single day of work
pub type TimePerTask = HashMap<Id, Duration>;

#[derive(Clone, Debug)]
pub struct WorkDay {
    working_hours: WorkingHours,
    time_per_task: TimePerTask,
}

impl WorkDay {
    /// Assign time equal to `duration` to the task `task` on this day
    ///
    /// # Panics
    /// Panics if passed a negative `Duration` of time
    // TODO This should use `std::Duration` or some other data structure that is non-negative
    pub fn add (&mut self, task: &Task, duration: Duration) {
        assert!(duration >= Duration::zero(), "Cannot add negative time to a workday!");

        let current_duration = self.time_per_task.entry(task.id).or_insert(Duration::zero());

        *current_duration = *current_duration + duration;
    }

    #[must_use] pub fn new (working_hours: WorkingHours) -> Self {
        Self {
            working_hours,
            time_per_task: TimePerTask::default(),
        }
    }

    /// Returns the amount of time still available on this day
    /// i.e., the number of working hours minus the time already assigned
    /// If more time has been assigned than is available, returns a 0 duration
    ///
    /// NOTE: This can be incorrect for today's date, since some time might have already passed
    #[must_use] pub fn raw_time_available (&self) -> Duration {
        max(Duration::zero(), self.working_hours.working_time() - self.time_assigned())
    }

    /// Pure
    #[must_use] pub fn time_assigned (&self) -> Duration {
        self.time_per_task
            .values()
            .sum()
    }
}

#[derive(Clone, Default, Debug)]
pub struct Schedule  {
    days_off: Vec<NaiveDate>,
    work_days: WorkDays,
    work_week: WorkWeek,
}

impl Schedule {
    /// Construct a `Schedule` with the passed `days_off` and `work_week`
    /// Calculates workloads in-place from `tasks`
    #[must_use] pub fn new (days_off: Vec<NaiveDate>, tasks: &Vec<Task>, work_week: WorkWeek) -> Self {
        let mut schedule = Schedule {
            days_off,
            work_days: WorkDays::new(),
            work_week,
        };

        schedule.assign_time_to_days(tasks);

        schedule
    }

    /// Impure (calls `today_date`)
    ///
    /// Return the amount of time left to work today
    /// If the current time is before the start time, return the total time available today
    /// Otherwise, return the time between now and the end of the day
    /// If there are no hours of work today, return None
    #[must_use] pub fn time_remaining_today(&self) -> Option<Duration> {
        self.work_week.days[&today_date().weekday()]
            .hours_of_work
            .map(|hour_range| max(
                Duration::zero(),
                hour_range.end - max(now_time(), hour_range.start)
            ))
    }

    /// Impure (calls `time_remaining_today`, `today_date`)
    ///
    // TODO This is a mess. Need to be consistent with when today_date() is called, i.e., which functions are pure 
    //      What should be implemented on `WorkDay` vs. on `Schedule`?
    fn time_available_today(&self) -> Option<Duration> {
        Some(max(Duration::zero(), self.time_remaining_today()? - self.get(today_date())?.time_assigned()))

    }

    /// Impure (calls `time_available_today`)
    ///
    /// Returns the amount of time still available on a date
    /// i.e., the number of working hours minus the number of hours of work assigned to the day
    #[must_use] pub fn time_available_on_date(&self, date: NaiveDate) -> Option<Duration> {
        if date == today_date() {
            self.time_available_today()
        } else {
            Some(self.get(date)?.raw_time_available())
        }
    }

    /// Pure
    ///
    /// Returns an iterator over the working days between two dates, including both ends
    fn work_days_from(&self, d1: NaiveDate, d2: NaiveDate) -> impl Iterator<Item = NaiveDate> + '_ {
        DateIterator::new(d1, Some(d2))
            .filter(|d| self.is_work_day(*d))
    }

    /// Impure (calls `last_available_date_for_task`, `first_available_date_for_task`)
    ///
    /// Returns an iterator over the days a task can be worked on, or nothing if the task has no due
    /// date (i.e., the range is undefined)
    fn work_days_for_task(&self, task: &Task) -> Option<Vec<NaiveDate>> {
        self.last_available_date_for_task(task)
            .map(|due_date| self.work_days_from(self.first_available_date_for_task(task), due_date).collect())
    }

    /// Impure (call `work_days_for_task`)
    ///
    /// Returns the number of days a task can be worked on, if there is a due date
    fn num_days_to_work_on(&self, task: &Task) -> Option<u32> {
        Some(self.work_days_for_task(task)?.len().try_into().expect("This fails on huge numbers"))
    }

    /// Impure (calls `today_date`)
    ///
    /// Returns the date of the soonest work day, including today
    fn next_work_day(&self) -> NaiveDate {
        let mut day = today_date();
        while !self.is_work_day(day) {
            day = day.succ_opt().expect("This will fail on huge dates");
        }

        day
    }

    /// Impure (calls `next_work_day`)
    ///
    /// Returns first date that a task can be worked on
    #[must_use] pub fn first_available_date_for_task(&self, task: &Task) -> NaiveDate {
        max(task.next_action_date, self.next_work_day())
    }

    /// Impure (calls `next_work_day`, `first_available_date_for_task`)
    ///
    /// Returns the last date that a task can be worked on
    #[must_use] pub fn last_available_date_for_task(&self, task: &Task) -> Option<NaiveDate> {
        match task.due_date {
            DueDate::Never => None,
            DueDate::Date(due_date) => Some(max(due_date, self.next_work_day())),
            DueDate::Asap => Some(self.first_available_date_for_task(task)),
        }
    }

    /// Calculates and records the number of minutes that need to be worked each day
    fn assign_time_to_days (&mut self, tasks: &Vec<Task>) {
        self.assign_time_by_frontloading_work(tasks);
    }

    /// One variant of the workload calculation
    /// This sorts the tasks from first due to last, and schedules work as early as possible
    // TODO a lot of this code counts on `Duration`s being positive, but the chrono `Duration` doesn't make this guarantee
    fn assign_time_by_frontloading_work (&mut self, tasks: &Vec<Task>) {

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
                    let workload_for_day = min(
                        self.time_available_on_date(day)
                            .expect("This will be Some because all work days have non-None time, and this loops over work days only"),
                        time_to_assign
                    );
                    // Remove the time to be allocated from the remaining time for the task
                    time_to_assign = time_to_assign - workload_for_day;

                    self.work_days
                        .entry(day)
                        .or_insert(WorkDay::new(WorkingHours::new(self.work_week.working_hours_on_day(day).hours_of_work)))
                        .add(task, workload_for_day);
                }

                // If time remains, assign to final day
                if time_to_assign.num_seconds() != 0 {
                    self.work_days.get_mut(
                            self.work_days_for_task(task)
                                .expect("We already checked this is not None")
                                .last()
                                .expect("work_days_for_task never returns an empty iterator")
                        )
                        .expect("This will not be None because we've already inserted a value in the previous loop")
                        .add(task, time_to_assign);
                }
            } else {
                // TODO TBD how to handle tasks that are not available for any days
            }
        }
    }

    /// Impure (calls `last_available_date_for_task`)
    ///
    /// Returns a boolean representing whether a given task can be worked on on a given date
    #[must_use] pub fn is_available_on_day(&self, task: &Task, date: NaiveDate) -> bool {
        let before_end = self.last_available_date_for_task(task).map_or(true, |available_date| date <= available_date);

        let after_beginning = task.next_action_date <= date;

        before_end && after_beginning
    }

    /// Pure
    ///
    /// Returns the duration of work that need to be done on a given date
    #[must_use] pub fn get_time_per_task_on_day(&self, date: NaiveDate) -> Option<&TimePerTask> {
        Some(
            &self.work_days
                .get(&date)? // The duration of each task assigned to the day
                .time_per_task
        )
    }

    /// Pure
    ///
    #[must_use] pub fn get_time_assigned_on_day(&self, date: NaiveDate) -> Option<Duration> {
        Some(self.get(date)?.time_assigned())
    }

    /// Pure
    ///
    /// Returns a boolean representing whether a given date is a work day
    #[must_use] pub fn is_work_day(&self, date: NaiveDate) -> bool {
        !self.days_off.contains(&date) && self.work_week.workdays().contains(&date.weekday())
    }

    /// Pure
    ///
    /// Returns the `WorkDay` at the date indicated
    /// If there is no work assigned to that `WorkDay`, returns an empty `WorkDay` with the correct
    /// hours of work.
    #[must_use] pub fn get (&self, date: NaiveDate) -> Option<WorkDay> {
        if self.is_work_day(date) && date >= today_date() {
            Some(self.work_days
                .get(&date)
                .unwrap_or(
                    &WorkDay {
                        time_per_task: TimePerTask::new(),
                        working_hours: self.work_week.working_hours_on_day(date)
                    }
                )
                .clone()
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkWeek {
    days: HashMap<Weekday, WorkingHours>
}

impl WorkWeek {
    /// Pure
    #[must_use] pub fn workdays(&self) -> Vec<Weekday> {
        self.days.iter()
            .filter(|(_, workday)| workday.working_time() > Duration::zero())
            .map(|(weekday, _)| *weekday)
            .collect()
    }

    /// Pure
    #[must_use] pub fn working_hours_on_day(&self, day: NaiveDate) -> WorkingHours {
        self.days[&day.weekday()]
    }
}

impl Default for WorkWeek {
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

        WorkWeek {days}
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WorkingHours {
    hours_of_work: Option<HourRange>,
}

impl WorkingHours {
    #[must_use] pub fn new(hours: Option<HourRange>) -> Self {
        Self {
            hours_of_work: hours,
        }
    }

    /// Pure
    #[must_use] pub fn working_time(&self) -> Duration {
        match self.hours_of_work {
            Some(hours) => hours.duration(),
            None => Duration::zero(),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct HourRange {
    start: NaiveTime,
    end:   NaiveTime,
}

impl HourRange {
    #[must_use] pub fn new(start: NaiveTime, end: NaiveTime) -> Option<Self> {
        if end > start {
            Some(Self {start, end})
        } else {
            None
        }
    }

    #[must_use] pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}
