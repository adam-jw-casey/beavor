use chrono::{
    NaiveDate,
    offset::Utc,
    DateTime,
};

use iced::widget::{
    column,
    Column,
    row,
    text,
    text_input,
    checkbox,
    button,
    Space,
    container,
    pick_list,
};

use iced::{
    Length,
    Alignment,
};

use iced_aw::helpers::date_picker;

use backend::{
    Task,
    DueDate,
    utils::today_date,
};

use crate::Message;

#[derive(Debug, Clone)]
pub enum UpdateDraftTask{
    Category        (String),
    Name            (String),
    TimeNeeded      (Result<u32, ()>),
    TimeUsed        (Result<u32, ()>),
    NextActionDate  (NaiveDate),
    DueDate         (DueDate),
    Notes           (String),
    Finished        (bool),
}

// TODO should have a dedicated State object to pass in so don't have to keep updating arguments
pub fn task_editor<'a>(task: &'a Task, timer_start_utc: Option<&'a DateTime<Utc>>, show_next_action_date_picker: bool, show_due_date_picker: bool) -> Column<'a, Message>{
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
        Space::new(Length::Fill, Length::Fill),
        row![
            text("Category").width(Length::FillPortion(1)),
            text_input(
                "Category...",
               &task.category
            )
                .on_input(|s| Message_UDT(UDT::Category(s)))
                .width(Length::FillPortion(3))
        ],
        row![
            text("Name").width(Length::FillPortion(1)),
            text_input(
                "Name...",
               &task.name
            )
                .on_input(|s| Message_UDT(UDT::Name(s)))
				.width(Length::FillPortion(3))
        ],
        row![
            text("Time needed").width(Length::FillPortion(1)),
            text_input(
                "Time needed...",
               &task.time_needed.to_string()
            )
                .on_input(|u| Message_UDT(UDT::TimeNeeded(u.parse().map_err(|_| ()))))
				.width(Length::FillPortion(3))
        ],
        row![
            text("Time used").width(Length::FillPortion(1)),
            text_input(
                "Time used...",
               &format!("{}", display_time_used/60),
            )
                .on_input(|u| Message_UDT(UDT::TimeUsed(u.parse().map_err(|_| ()))))
				.width(Length::FillPortion(3))
        ],
        row![
            text("Next action")
                .width(Length::FillPortion(1)),
            container(
                date_picker(
                    show_next_action_date_picker,
                    task.next_action_date,
                    button(text(&task.next_action_date.to_string())).on_press(Message::PickNextActionDate),
                    Message::CancelPickNextActionDate,
                    |d| Message_UDT(UDT::NextActionDate(d.into()))
                )
            )
                .width(Length::FillPortion(3)),
        ],
        row![
            text("Due date").width(Length::FillPortion(1)),
            container(
                date_picker(
                    show_due_date_picker,
                    match task.due_date{
                        DueDate::Date(date) => date,
                        _ => today_date(), // This will not be shown, so arbitray
                    },
                    button(text(&task.due_date.to_string()))
                        .on_press_maybe(match task.due_date{
                            DueDate::Date(_) => Some(Message::PickDueDate),
                            _ => None,
                        }),
                    Message::CancelPickDueDate,
                    |d| Message_UDT(UDT::DueDate(DueDate::Date(d.into())))
                )
            )
                .width(Length::FillPortion(2)),
            pick_list( // TODO this whole section would be easier if DueDateType had to_string()
                vec!["Date", "None", "ASAP"],
                Some(match task.due_date{
                    DueDate::Never => "None",
                    DueDate::Date(_) => "Date",
                    DueDate::Asap => "ASAP",
                }),
                |selection| {
                    Message_UDT(UDT::DueDate(match selection{
                        "None" => DueDate::Never,
                        "ASAP" => DueDate::Asap,
                        "Date" => DueDate::Date(today_date()),
                        _ => panic!("This will never happen")
                    }))
                }
            )
                .width(Length::FillPortion(1)),
        ],
        row![
            text("Notes").width(Length::FillPortion(1)),
            text_input(
                "Notes...",
               &task.notes
            )
                .on_input(|d| Message_UDT(UDT::Notes(d)))
				.width(Length::FillPortion(3))
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
            .align_items(Alignment::Center)
            .spacing(4),
    ]
        .spacing(4)
        .align_items(Alignment::Center)
}
