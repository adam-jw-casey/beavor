use std::fmt::Display;

use chrono::NaiveDate;

use iced::widget::{
    Column,
    column,
    scrollable,
    text,
    button,
    row,
    rule::Rule,
    space::Space,
};

use iced::{
    Element,
    Length,
    Alignment,
};

use backend::{
    Task,
    Schedule,
};

use crate::{
    Message,
    CalendarMessage,
};

trait Filter: Display { // TODO rather than display, should really impl Into<Element<'static, Message>>
    fn apply(&self, task: &Task) -> bool;
    fn cancel(&self) -> Message;
}

struct DateFilter<'date, 'schedule> {
    date: &'date NaiveDate,
    schedule: &'schedule Schedule,
}

impl Filter for DateFilter<'_, '_> {
    fn apply(&self, t: &Task) -> bool {
        self.schedule.is_available_on_day(
            t,
            *self.date
        )
    }

    fn cancel(&self) -> Message {
        Message::Calendar(CalendarMessage::FilterToDate(None))
    }
}

impl Display for DateFilter<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Available on: {}", self.date.format("%b %d"))
    }
}

pub fn task_scroller(tasks: &[Task], filter_date: Option<&NaiveDate>, schedule: &Schedule) -> Column<'static, Message> {

    let filters = [ // TODO this should probably be in the application-level state
        filter_date.map(|date| DateFilter {
            date,
            schedule
        }),
    ];

    column![
        Column::with_children(
            filters
                .iter()
                .filter_map(|f| f.as_ref().map(|f|
                    row![
                        button("X").on_press(f.cancel()),
                        text(f.to_string())
                    ]
                    .align_items(Alignment::Center)
                    .spacing(4)
                    .into())
                )
                .chain(
                    [if filters.iter().any(Option::is_some) {
                        Rule::horizontal(2).into()
                    }else {
                        Space::with_height(0).into()
                    }]
                )
                .collect()
        ).spacing(4),
        scrollable(
            Column::with_children(
                tasks
                    .iter()
                    .filter(|t|
                        filters
                            .iter()
                            .map(|f| match f {
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
        )
            .height(Length::Fill),
    ]
        .spacing(4)
        .padding(4)
}

fn task_row(task: &Task) -> Element<'static, Message> {
    button(
        column![
            text(&task.name),
            text(&task.category),
        ]
    )
        .on_press(Message::TrySelectTask(Some(task.clone())))
        .width(Length::Fill)
        .into()
}
