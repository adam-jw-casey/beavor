use chrono::NaiveDate;

use iced::widget::{
    Column,
    column,
    scrollable,
    Scrollable,
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

pub fn task_scroller(tasks: &[Task], filter_date: Option<&NaiveDate>, schedule: &Schedule) -> Scrollable<'static, Message> {

    scrollable(
        Column::with_children(
            tasks
                .iter()
                .filter(|t| match filter_date{
                    None => true,
                    Some(date) => {
                        schedule.is_available_on_day(
                            t,
                            *date
                        )
                    }
                })
                .map(task_row)
                .collect()
        )
            .width(Length::Shrink)
            .spacing(2)
            .padding(4)
    )
        .height(Length::Fill)
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
