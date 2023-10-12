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

use chrono::{
    offset::Utc,
    DateTime,
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
pub fn TaskEditor(task: &Task, timer_start_utc: Option<&DateTime<Utc>>) -> Element<'static, Message>{
    use Message::UpdateDraftTask as Message_UDT;
    use UpdateDraftTask as UDT;

    let display_time_used: u32 = task.time_used * 60 + match timer_start_utc{
        Some(timer_start_utc) => (Utc::now() - timer_start_utc).num_seconds() as u32, 
            // This should work because the timer presumably started in the past, which will
            // yield a positive number, and presumably has not been running long enough to
            // overflow a u32 (136 years by my math)
        None => 0,
    };

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
               &format!("{}", display_time_used/60),
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
                match timer_start_utc{
                    Some(_) => "Stop",
                    None => "Start",
                }
            )
                .on_press(Message::ToggleTimer),
            text(
                 format!("{:02}:{:02}:{:02}", display_time_used/3600, (display_time_used % 3600)/60, display_time_used % 60)
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
