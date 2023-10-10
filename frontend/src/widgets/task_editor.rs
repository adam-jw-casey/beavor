use chrono::NaiveDate;

use iced::widget::{
    column,
    row,
    text,
    text_input,
};

use iced::{
    Element,
    Length,
};

use backend::{
    Task,
    DueDate,
};

use crate::Message;

#[derive(Debug, Clone)]
pub enum UpdateDraftTask{
    Category        (String),
    Name            (String),
    TimeNeeded      (u32),
    TimeUsed        (u32),
    NextActionDate      (NaiveDate),
    DueDate         (DueDate),
    Notes           (String),
}

#[allow(non_snake_case)]
pub fn TaskEditor(task: Option<&Task>) -> Element<'static, Message>{
    column![
        row![
            text("Category"),
            text_input(
                "Category...",
               &task.unwrap_or(&Task::default()).category
            )
                .on_input(|s| Message::UpdateDraftTask(UpdateDraftTask::Category(s)))
                .width(Length::Fill)
        ],
        row![
            text("Name"),
            text_input(
                "Name...",
               &task.unwrap_or(&Task::default()).name
            )
                .on_input(|s| Message::UpdateDraftTask(UpdateDraftTask::Name(s)))
				.width(Length::Fill)
        ],
        row![
            text("Time needed"),
            text_input(
                "Time needed...",
               &task.unwrap_or(&Task::default()).time_needed.to_string()
            )
                .on_input(|u| Message::UpdateDraftTask(UpdateDraftTask::TimeNeeded(u.parse().expect("Should parse")))) // TODO I have a feeling all this parsing would be better handled at the application level so that an error modal can be shown or something
				.width(Length::Fill)
        ],
        row![
            text("Time used"),
            text_input(
                "Time used...",
               &task.unwrap_or(&Task::default()).time_used.to_string()
            )
                .on_input(|u| Message::UpdateDraftTask(UpdateDraftTask::TimeUsed(u.parse().expect("Should parse"))))
				.width(Length::Fill)
        ],
        row![
            text("Next action"),
            text_input(
                "Next action...",
               &task.unwrap_or(&Task::default()).next_action_date.to_string()
            )
                .on_input(|d| Message::UpdateDraftTask(UpdateDraftTask::NextActionDate(d.parse().expect("Should parse"))))
				.width(Length::Fill)
        ],
        row![
            text("Due date"),
            text_input(
                "Due date...",
               &task.unwrap_or(&Task::default()).next_action_date.to_string()
            )
                .on_input(|d| Message::UpdateDraftTask(UpdateDraftTask::DueDate(d.parse().expect("Should parse"))))
				.width(Length::Fill)
        ],
        row![
            text("Notes"),
            text_input(
                "Notes...",
               &task.unwrap_or(&Task::default()).notes
            )
                .on_input(|d| Message::UpdateDraftTask(UpdateDraftTask::Notes(d)))
				.width(Length::Fill)
        ],
    ]
        .width(Length::FillPortion(1))
        .into()
}
