use iced::widget::{
    container,
    row,
};


use iced::{
    Application,
    Element,
    Settings,
    Subscription,
    Command,
    executor,
    Theme,
    time::Instant,
    Length,
    Alignment,
};

use chrono::{
    NaiveDate,
    offset::Utc,
    DateTime,
};

use backend::{
    DatabaseManager,
    Task,
};

mod widgets;
use widgets::{
    calendar,
    task_scroller,
    task_editor::task_editor,
};

use widgets::task_editor::UpdateDraftTask;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
pub enum Message{
    Tick(Instant),
    SelectTask(Option<Task>),
    SelectDate(Option<NaiveDate>),
    UpdateDraftTask(UpdateDraftTask),
    SaveDraftTask,
    NewTask,
    DeleteTask,
    ToggleTimer, // Consider having seperate start/stop/toggle messages
    PickNextActionDate,
    CancelPickNextActionDate,
    PickDueDate,
    CancelPickDueDate,
}

// TODO need a better way of keeping track of whether the shown task:
//          a. Already exists in the database, as shown,
//          b. Exists in the database, but is shown with unsaved user modifications,
//          c. Is the default, placeholder task, or
//          d. Is a new task with some or all information already entered
//  TODO Whatever type draft_task ends of having, it should have a take/replace/swap like in
//       std::mem to get an owned copy and overwrite the original in an "atomic" operation, to
//       help ease state management
struct Beavor{
    db:            DatabaseManager,
    selected_task: Option<Task>, // TODO should this maybe just store task ID to minimize the state
                                 // lying around?
    selected_date: Option<NaiveDate>,
    draft_task: Task,
    timer_start_utc: Option<DateTime<Utc>>, // This should really be a custom enum like TimerState
                                            // or somesuch
                                            // It should also have a method to compute the time
                                            // since started, since this is done in several places
    next_action_date_picker_showing: bool,
    due_date_picker_showing: bool,
}

impl Application for Beavor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Beavor, iced::Command<Message>) {
        // TODO database path should be a flag
        let db = DatabaseManager::new("worklist.db".into())
            .unwrap_or_else(|_| DatabaseManager::create_new_database("worklist.db".into()).expect("Should be able to create database"));
        (
            Self{
                db,
                selected_task: None,
                selected_date: None,
                draft_task:    Task::default(),
                timer_start_utc: None,
                next_action_date_picker_showing: false,
                due_date_picker_showing: false,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    // TODO break each match case out into seperate functions (or at least into groups). This is getting ridiculous.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message{
            Message::SelectTask(maybe_task) => {
                // Don't overwrite an existing draft task
                if (self.selected_task.is_none() && self.draft_task == Task::default()) || self.draft_task == *self.selected_task.as_ref().unwrap(){
                    self.selected_task = maybe_task.clone();
                    self.draft_task = match maybe_task{
                        Some(t) =>  t.clone(),
                        None => Task::default(),
                    }
                }else{
                    println!("Refusing to overwrite draft task"); // TODO handle this case
                                                                  // elegantly
                }
            },
            Message::SelectDate(maybe_date) => self.selected_date = maybe_date,
            Message::UpdateDraftTask(task_field_update) => {
                let t = &mut self.draft_task;
                use UpdateDraftTask as UDT;
                match task_field_update{
                    UDT::Category(category) => t.category = category,
                    UDT::Name(name) => t.name = name,
                    UDT::TimeNeeded(time_needed) => if let Ok(time_needed) = time_needed {t.time_needed = time_needed},
                    UDT::TimeUsed(time_used) => if let Ok(time_used) = time_used {t.time_used = time_used},
                    UDT::NextActionDate(next_action_date) => {
                        t.next_action_date = next_action_date;
                        let _ = self.update(Message::CancelPickNextActionDate);
                    },
                    UDT::DueDate(due_date) => {
                        t.due_date = due_date;
                        let _ = self.update(Message::CancelPickDueDate);
                    },
                    UDT::Notes(notes) => t.notes = notes,
                    UDT::Finished(finished) => t.finished = finished,
                }
            },
            Message::SaveDraftTask => {
                let t = &self.draft_task;
                match t.id{
                    Some(_) => self.db.update_task(t),
                    None => {
                        self.draft_task = self.db.create_task(t);
                    },
                }
                self.selected_task = Some(self.draft_task.clone());
            },
            Message::NewTask => {let _ = self.update(Message::SelectTask(None));},
            Message::DeleteTask => {
                let t = std::mem::take(&mut self.draft_task);
                self.db.delete_task(t);
                self.selected_task = None;
                let _ = self.update(Message::NewTask);
            },
            Message::ToggleTimer => match self.timer_start_utc{
                Some(timer_start_utc) => {
                    self.draft_task.time_used += (Utc::now() - timer_start_utc).num_minutes() as u32;
                    self.timer_start_utc = None;
                    let _ = self.update(Message::SaveDraftTask);
                },
                None => self.timer_start_utc = Some(Utc::now()),
            },
            Message::Tick(_) => {},
            Message::PickNextActionDate => self.next_action_date_picker_showing = true,
            Message::CancelPickNextActionDate => self.next_action_date_picker_showing = false,
            Message::PickDueDate => self.due_date_picker_showing = true,
            Message::CancelPickDueDate => self.due_date_picker_showing = false,
        };
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = row![
            task_scroller(&self.db.get_open_tasks())
                .width(Length::FillPortion(2))
                .height(Length::FillPortion(1)),
            task_editor(&self.draft_task, self.timer_start_utc.as_ref(), self.next_action_date_picker_showing, self.due_date_picker_showing)
                .padding(8)
                .width(Length::FillPortion(3))
                .height(Length::FillPortion(1)),
            calendar(&self.db.get_schedule()),
        ]
            .align_items(Alignment::End)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();

        container(content).into()
    }

    fn subscription(&self) -> Subscription<Self::Message>{
        iced::time::every(iced::time::Duration::from_secs(1)).map(Message::Tick)
    }
}
