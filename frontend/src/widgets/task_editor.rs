use chrono::{
    NaiveDate,
    offset::Utc,
    DateTime,
    Duration,
};

use iced::widget::{
    column,
    Column,
    row,
    Row,
    text,
    text_input,
    checkbox,
    button,
    Space,
    container,
    Container,
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
    Hyperlink,
};

use crate::{
    Message,
    MutateMessage,
    ModalMessage,
    widgets::hyperlink,
    ModalShowing,
};

// TODO this still doesn't sit quite right, since why match over TimerState when you can match over
// time_running and immediately get the time?
#[derive(Debug, Clone)]
pub enum TimerState{
    Timing{
        start_time: DateTime<Utc>,
    },
    Stopped,
}

impl TimerState{
    pub fn time_running(&self) -> Option<Duration>{
        match self{
            TimerState::Timing { start_time } => Some(Utc::now() - start_time),
            TimerState::Stopped => None,
        }
    }

    pub fn num_minutes_running(&self) -> Option<u32> {
        Some(u32::try_from(self.time_running()?.num_minutes())
            .expect("This will be positive (started in the past) and small enough to fit (<136 years)"))
    }

    pub fn num_seconds_running(&self) -> Option<u32> {
        Some(u32::try_from(self.time_running()?.num_seconds())
            .expect("This will be positive (started in the past) and small enough to fit (<136 years)"))
    }
}

#[derive(Debug, Clone)]
pub enum LinkMessage{
    New,
    Delete(usize),
    Update((Hyperlink, usize)),
}

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
    Link            (LinkMessage),
}

use Message::UpdateDraftTask as Message_UDT;
use UpdateDraftTask as UDT;

// TODO Should buttons be disabled while a date modal is open? Or should clicking one of them
// close the modal?
// TODO should have a dedicated State object to pass in so don't have to keep updating arguments
pub fn task_editor<'a>(draft_task: &'a Task, timer_state: &TimerState, date_picker_state: &ModalShowing, editing_link: Option<usize>) -> Column<'a, Message>{

    let display_time_used: u32 = draft_task.time_used * 60 + timer_state.num_seconds_running().unwrap_or(0);

    // TODO this is a TON of boilerplate. Find a way to reduce this down
    column![
        Space::new(Length::Fill, Length::Fill),
        row![
            text("Category").width(Length::FillPortion(1)),
            text_input(
                "Category...",
               &draft_task.category
            )
                .on_input(|s| Message_UDT(UDT::Category(s)))
                .width(Length::FillPortion(3))
        ],
        row![
            text("Name").width(Length::FillPortion(1)),
            text_input(
                "Name...",
               &draft_task.name
            )
                .on_input(|s| Message_UDT(UDT::Name(s)))
				.width(Length::FillPortion(3))
        ],
        row![
            text("Time needed").width(Length::FillPortion(1)),
            text_input(
                "Time needed...",
               &draft_task.time_needed.to_string()
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
            next_action_date_picker(date_picker_state, draft_task)
                .width(Length::FillPortion(3)),
        ],
        row![
            text("Due date").width(Length::FillPortion(1)),
            due_date_picker(date_picker_state, draft_task)
                .width(Length::FillPortion(3)),
        ],
        Column::with_children(
            (0..draft_task.links.len())
                .map(|idx: usize| {
                    hyperlink(&draft_task.links[idx], idx, editing_link)
                })
                .collect()
        )
            .spacing(4),
        row![
            Space::with_width(Length::Fill),
            button("Add link")
                .on_press_maybe(
                    if editing_link.is_none(){
                        Some(Message_UDT(UDT::Link(LinkMessage::New)))
                    }else{None}
                )
        ],
        row![
            text("Notes").width(Length::FillPortion(1)),
            text_input(
                "Notes...",
               &draft_task.notes
            )
                .on_input(|d| Message_UDT(UDT::Notes(d)))
				.width(Length::FillPortion(3))
        ],
        row![
            checkbox(
                "Done",
                draft_task.finished,
                |b| Message_UDT(UDT::Finished(b)),
            ),
            button(
                match timer_state{
                    TimerState::Timing{..} => "Stop",
                    TimerState::Stopped => "Start",
                }
            ).on_press(Message::ToggleTimer),
            text( format!("{:02}:{:02}:{:02}", display_time_used/3600, (display_time_used % 3600)/60, display_time_used % 60)),
            button("Save").on_press(Message::Mutate(MutateMessage::SaveDraftTask)),
            button("New").on_press(Message::NewTask),
            // TODO this should be disabled if the current task is a new one (i.e., does not exist
            // in the database
            button("Delete").on_press(Message::TryDeleteTask),
        ]
            .align_items(Alignment::Center)
            .spacing(4),
    ]
        .spacing(4)
        .align_items(Alignment::Center)
}

fn next_action_date_picker<'a>(date_picker_state: &ModalShowing, draft_task: &'a Task) -> Container<'a, Message>{
    container(
        date_picker(
            matches!(date_picker_state, ModalShowing::NextAction),
            draft_task.next_action_date,
            button(text(&draft_task.next_action_date.to_string())).on_press(Message::Modal(ModalMessage::PickNextActionDate)),
            Message::Modal(ModalMessage::Close),
            |d| Message_UDT(UDT::NextActionDate(d.into()))
        )
    )
}

fn due_date_picker<'a>(date_picker_state: &ModalShowing, draft_task: &'a Task) -> Row<'a, Message>{
    row![
        container(
            date_picker(
                matches!(date_picker_state, ModalShowing::DueDate),
                match draft_task.due_date{
                    DueDate::Date(date) => date,
                    _ => today_date(), // This will not be shown, so arbitray
                },
                button(text(&draft_task.due_date.to_string()))
                    .on_press_maybe(match draft_task.due_date{
                        DueDate::Date(_) => Some(Message::Modal(ModalMessage::PickDueDate)),
                        _ => None,
                    }),
                Message::Modal(ModalMessage::Close),
                |d| Message_UDT(UDT::DueDate(DueDate::Date(d.into())))
            )
        ).width(Length::FillPortion(1)),
        pick_list(
            vec!["Date", "None", "ASAP"],
            Some(match draft_task.due_date{
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
        ).width(Length::FillPortion(2))
    ]
}
