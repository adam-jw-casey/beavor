use chrono::NaiveDate;

use iced::widget::{
    column,
    row,
    text,
    text_input,
    checkbox,
    button,
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
    NextActionDate  (NaiveDate),
    DueDate         (DueDate),
    Notes           (String),
    Finished        (bool),
}

#[allow(non_snake_case)]
pub fn TaskEditor(task: &Task) -> Element<'static, Message>{
    use Message::UpdateDraftTask as Message_UDT;
    use UpdateDraftTask as UDT;
    // TODO this is a TON of boilerplate. Find a way to reduce this down
    column![
        row![
            text("Category"),
            text_input(
                "Category...",
               &task.category
            )
                .on_input(|s| Message_UDT(UDT::Category(s)))
                .width(Length::Fill)
        ],
        row![
            text("Name"),
            text_input(
                "Name...",
               &task.name
            )
                .on_input(|s| Message_UDT(UDT::Name(s)))
				.width(Length::Fill)
        ],
        row![
            text("Time needed"),
            text_input(
                "Time needed...",
               &task.time_needed.to_string()
            )
                .on_input(|u| Message_UDT(UDT::TimeNeeded(u.parse().expect("Should parse")))) // TODO I have a feeling all this parsing would be better handled at the application level so that an error modal can be shown or something
				.width(Length::Fill)
        ],
        row![
            text("Time used"),
            text_input(
                "Time used...",
               &task.time_used.to_string()
            )
                .on_input(|u| Message_UDT(UDT::TimeUsed(u.parse().expect("Should parse"))))
				.width(Length::Fill)
        ],
        row![
            text("Next action"),
            text_input(
                "Next action...",
               &task.next_action_date.to_string()
            )
                .on_input(|d| Message_UDT(UDT::NextActionDate(d.parse().expect("Should parse"))))
				.width(Length::Fill)
        ],
        row![
            text("Due date"),
            text_input(
                "Due date...",
               &task.next_action_date.to_string()
            )
                .on_input(|d| Message_UDT(UDT::DueDate(d.parse().expect("Should parse"))))
				.width(Length::Fill)
        ],
        row![
            text("Notes"),
            text_input(
                "Notes...",
               &task.notes
            )
                .on_input(|d| Message_UDT(UDT::Notes(d)))
				.width(Length::Fill)
        ],
        row![
            checkbox(
                "Done",
                task.finished,
                |b| Message_UDT(UDT::Finished(b)),
            ),
            button(
                "Start",
            )
                .on_press(Message::ToggleTimer),
            text(
                "0:00:00"
            ),
            button(
                "Save",
            )
                .on_press(Message::SaveDraftTask),
            button(
                "New",
            )
                .on_press(Message::NewTask),
            button(
                "Delete",
            )
                .on_press(Message::DeleteTask),
        ]
    ]
        .width(Length::FillPortion(1))
        .into()
}
