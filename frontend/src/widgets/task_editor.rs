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
    ComboBox,
    combo_box::State as ComboBoxState,
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
};

use Message::UpdateDraftTask as Message_UDT;
use UpdateDraftTask as UDT;

// time_running and immediately get the time?
#[derive(Debug, Clone)]
pub enum TimerState{
    Timing{
        start_time: DateTime<Utc>,
    },
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub enum TimerMessage{
    Start,
    Stop,
    Toggle,
}

impl Default for TimerState{
    fn default() -> Self {
        Self::Stopped
    }
}

impl TimerState{
    pub fn time_running(&self) -> Option<Duration>{
        match self{
            TimerState::Timing { start_time } => Some(Utc::now() - start_time),
            TimerState::Stopped => None,
        }
    }

    pub fn num_seconds_running(&self) -> Option<u32> {
        Some(u32::try_from(self.time_running()?.num_seconds())
            .expect("This will be positive (started in the past) and small enough to fit (<136 years)"))
    }

    pub fn start(&mut self){
        if matches!(self, TimerState::Stopped){
            *self = TimerState::Timing{start_time: Utc::now()};
        }
    }

    pub fn stop(&mut self) -> Option<Duration>{
        if let Some(minutes) = self.time_running(){
            *self = TimerState::Stopped;
            Some(minutes)
        }else{
            None
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DisplayedTask{
    selected:               Option<Task>,
    pub draft:              Task,
    pub editing_link_idx:   Option<usize>,
    pub timer:              TimerState,
}

impl DisplayedTask{
    pub fn added_time(&self) -> Option<Duration> {
        Some(self.draft.time_used - self.selected.clone()?.time_used)
    }

    pub fn is_unmodified(&self) -> bool{
        match &self.selected{
            Some(t) => *t == self.draft,
            None => self.draft == Task::default(),
        }
    }

    pub fn select(&mut self, maybe_task: Option<Task>){
        self.selected = maybe_task.clone();
        self.draft = match maybe_task{
            Some(t) =>  t.clone(),
            None => Task::default(),
        };
    }

    pub fn stop_timer(&mut self){
        if let Some(duration) = self.timer.stop() {
            self.draft.time_used = self.draft.time_used + duration;
        }
    }

    pub fn update_timer(&mut self, message: TimerMessage) {
        match message{
            TimerMessage::Start => self.timer.start(),
            TimerMessage::Stop => self.stop_timer(),
            TimerMessage::Toggle => match self.timer{
                TimerState::Timing{..} => self.update_timer(TimerMessage::Stop),
                TimerState::Stopped => self.update_timer(TimerMessage::Start),
            },
        }
    }

    // This warning occurs because of the unreachable `panic!()` below
    #[allow(clippy::missing_panics_doc)]
    #[must_use] pub fn update_draft(&mut self, message: UpdateDraftTask) -> Option<ModalType>{
        match message{
            UDT::NextActionDate(next_action_date) => {
                self.draft.next_action_date = next_action_date;
                Some(ModalType::None)
            },
            UDT::DueDate(due_date) => {
                self.draft.due_date = due_date;
                Some(ModalType::None)
            },
            other => {
                match other{
                    UDT::NextActionDate(_) | UDT::DueDate(_) => panic!("This will never happen"),
                    UDT::Category(category) => self.draft.category = category,
                    UDT::Name(name) => self.draft.name = name,
                    UDT::TimeNeeded(time_needed) => if let Ok(time_needed) = time_needed {self.draft.time_needed = Duration::minutes(time_needed.into())},
                    UDT::TimeUsed(time_used) => if let Ok(time_used) = time_used {self.draft.time_used = Duration::minutes(time_used.into())},
                    UDT::Notes(notes) => self.draft.notes = notes,
                    UDT::Finished(finished) => self.draft.finished = finished,
                    UDT::Link(link_message) => match link_message{
                        LinkMessage::New => if !self.draft.links.contains(&Hyperlink::default()){
                            self.draft.links.push(Hyperlink::default());
                            self.editing_link_idx = Some(self.draft.links.len()-1);

                        },
                        LinkMessage::Delete(idx) => {
                            self.draft.links.remove(idx);
                        },
                        LinkMessage::Update((link, idx)) => {
                            self.draft.links[idx] = link;
                        },
                    }
                }
                None
            }
        }
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

#[allow(clippy::too_many_lines)]
pub fn task_editor<'a, 'b>(displayed_task: &'a DisplayedTask, modal_state: &ModalType, combo_box_state: &'b ComboBoxState<String>) -> Column<'a, Message>
where 'b: 'a
{

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let display_time_used: u32 = displayed_task.draft.time_used.num_seconds() as u32 + displayed_task.timer.num_seconds_running().unwrap_or(0);

    // TODO this is a TON of boilerplate. Find a way to reduce this down
    column![
        Space::new(Length::Fill, Length::Fill),
        row![
            text("Category").width(Length::FillPortion(1)),
            ComboBox::new(
                combo_box_state,
                "Category...",
                Some(&displayed_task.draft.category),
                |s| Message_UDT(UDT::Category(s.to_string()))
            )
                .on_input(|s| Message_UDT(UDT::Category(s.to_string())))
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
               &displayed_task.draft.time_needed.num_minutes().to_string()
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
            ).on_press(Message::Timer(TimerMessage::Toggle)),
            text( format!("{:02}:{:02}:{:02}", display_time_used/3600, (display_time_used % 3600)/60, display_time_used % 60)),
            button("Save").on_press_maybe(
                if displayed_task.draft == Task::default(){
                    None
                }else{
                    Some(Message::Mutate(MutateMessage::SaveDraftTask))
                }
            ),
            button("New").on_press(Message::TryNewTask),
            button("Delete").on_press_maybe(
                if displayed_task.draft == Task::default(){
                    None
                }else{
                    Some(Message::TryDeleteTask)
                }
            ),
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
        pick_list( // TODO this code is unpleasing. This should really be something on DueDate
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
