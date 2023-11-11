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
    ModalType,
    DisplayedTask,
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
pub fn task_editor<'a>(displayed_task: &'a DisplayedTask, modal_state: &ModalType) -> Column<'a, Message>{

    let display_time_used: u32 = displayed_task.draft.time_used * 60 + displayed_task.timer.num_seconds_running().unwrap_or(0);

    // TODO this is a TON of boilerplate. Find a way to reduce this down
    column![
        Space::new(Length::Fill, Length::Fill),
        row![
            text("Category").width(Length::FillPortion(1)),
            text_input(
                "Category...",
               &displayed_task.draft.category
            )
                .on_input(|s| Message_UDT(UDT::Category(s)))
                .width(Length::FillPortion(3))
        ],
        row![
            text("Name").width(Length::FillPortion(1)),
            text_input(
                "Name...",
               &displayed_task.draft.name
            )
                .on_input(|s| Message_UDT(UDT::Name(s)))
				.width(Length::FillPortion(3))
        ],
        row![
            text("Time needed").width(Length::FillPortion(1)),
            text_input(
                "Time needed...",
               &displayed_task.draft.time_needed.to_string()
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
            next_action_date_picker(modal_state, &displayed_task.draft)
                .width(Length::FillPortion(3)),
        ],
        row![
            text("Due date").width(Length::FillPortion(1)),
            due_date_picker(modal_state, &displayed_task.draft)
                .width(Length::FillPortion(3)),
        ],
        Column::with_children(
            (0..displayed_task.draft.links.len())
                .map(|idx: usize| {
                    hyperlink(&displayed_task.draft.links[idx], idx, displayed_task.editing_link_idx)
                })
                .collect()
        )
            .spacing(4),
        row![
            Space::with_width(Length::Fill),
            button("Add link")
                .on_press_maybe(
                    if displayed_task.editing_link_idx.is_none(){
                        Some(Message_UDT(UDT::Link(LinkMessage::New)))
                    }else{None}
                )
        ],
        row![
            text("Notes").width(Length::FillPortion(1)),
            text_input(
                "Notes...",
               &displayed_task.draft.notes
            )
                .on_input(|d| Message_UDT(UDT::Notes(d)))
				.width(Length::FillPortion(3))
        ],
        row![
            checkbox(
                "Done",
                displayed_task.draft.finished,
                |b| Message_UDT(UDT::Finished(b)),
            ),
            button(
                match displayed_task.timer{
                    TimerState::Timing{..} => "Stop",
                    TimerState::Stopped => "Start",
                }
            ).on_press(Message::ToggleTimer),
            text( format!("{:02}:{:02}:{:02}", display_time_used/3600, (display_time_used % 3600)/60, display_time_used % 60)),
            button("Save").on_press(Message::Mutate(MutateMessage::SaveDraftTask)),
            button("New").on_press(Message::TryNewTask),
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

fn next_action_date_picker<'a>(modal_state: &ModalType, draft_task: &'a Task) -> Container<'a, Message>{
    container(
        date_picker(
            matches!(modal_state, ModalType::NextAction),
            draft_task.next_action_date,
            button(text(&draft_task.next_action_date.to_string())).on_press(Message::Modal(ModalMessage::Show(ModalType::NextAction))),
            Message::Modal(ModalMessage::Show(ModalType::None)),
            |d| Message_UDT(UDT::NextActionDate(d.into()))
        )
    )
}

fn due_date_picker<'a>(modal_state: &ModalType, draft_task: &'a Task) -> Row<'a, Message>{
    row![
        container(
            date_picker(
                matches!(modal_state, ModalType::DueDate),
                match draft_task.due_date{
                    DueDate::Date(date) => date,
                    _ => today_date(), // This will not be shown, so arbitray
                },
                button(text(&draft_task.due_date.to_string()))
                    .on_press_maybe(match draft_task.due_date{
                        DueDate::Date(_) => Some(Message::Modal(ModalMessage::Show(ModalType::DueDate))),
                        _ => None,
                    }),
                Message::Modal(ModalMessage::Show(ModalType::None)),
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
