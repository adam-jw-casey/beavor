use std::fmt::Display;

use chrono::NaiveDate;

use iced::widget::{
    Column,
    column,
    scrollable,
    text,
    Button,
};

use iced::{
    Element,
    Length,
};

use backend::{
    Task,
    Schedule,
};

use crate::Message;

trait Filter: Display{
    fn apply(&self, task: &Task) -> bool;
}

struct DateFilter<'a, 'b>{
    date: &'a NaiveDate,
    schedule: &'b Schedule,
}

impl Filter for DateFilter<'_, '_>{
    fn apply(&self, t: &Task) -> bool{
        self.schedule.is_available_on_day(
            t,
            *self.date
        )
    }
}

impl Display for DateFilter<'_, '_>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date.format("%b %d"))
    }
}

pub fn task_scroller(tasks: &[Task], filter_date: Option<&NaiveDate>, schedule: &Schedule) -> Column<'static, Message>{

    let filters = [ // TODO this should probably be in the application-level state
        filter_date.map(|date| DateFilter{
            date,
            schedule
        }),
    ];

    column![
        scrollable(
            Column::with_children(
                tasks
                    .iter()
                    .filter(|t|
                        filters
                            .iter()
                            .map(|f| match f{
                                None => true,
                                Some(f) => f.apply(t)
                            })
                            .all(|b| b)
                    )
                    .map(task_row)
                    .collect()
            )
                .width(Length::Shrink)
                .spacing(2)
                .padding(4)
        )
            .height(Length::Fill),
    ]
}

fn task_row(task: &Task) -> Element<'static, Message>{
    Button::new(
        column![
            text(&task.name),
            text(&task.category),
        ]
    )
        .on_press(Message::TrySelectTask(Some(task.clone())))
        .width(Length::Fill)
        .into()
}
