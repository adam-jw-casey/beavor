use iced::widget::{
    container,
    row,
};

use iced::{
    Sandbox,
    Element,
    Settings,
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
    Calendar,
    TaskScroller,
    TaskEditor,
};

use widgets::task_editor::UpdateDraftTask;

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
pub enum Message{
    SelectTask(Option<Task>),
    SelectDate(Option<NaiveDate>),
    UpdateDraftTask(UpdateDraftTask),
    SaveDraftTask,
    NewTask,
    DeleteTask,
    ToggleTimer, // Consider having seperate start/stop/toggle messages
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
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        let db = DatabaseManager::new("worklist.db".into());
        Self{
            db,
            selected_task: None,
            selected_date: None,
            draft_task:    Task::default(),
            timer_start_utc: None,
        }
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
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
                    UDT::TimeNeeded(time_needed) => t.time_needed = time_needed,
                    UDT::TimeUsed(time_used) => t.time_used = time_used,
                    UDT::NextActionDate(next_action_date) => t.next_action_date = next_action_date,
                    UDT::DueDate(due_date) => t.due_date = due_date,
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
            Message::NewTask => self.update(Message::SelectTask(None)),
            Message::DeleteTask => {
                let t = std::mem::take(&mut self.draft_task);
                self.db.delete_task(t);
                self.selected_task = None;
                self.update(Message::NewTask);
            },
            Message::ToggleTimer => match self.timer_start_utc{
                Some(timer_start_utc) => {
                    self.draft_task.time_used += (Utc::now() - timer_start_utc).num_minutes() as u32;
                    self.timer_start_utc = None;
                    self.update(Message::SaveDraftTask)
                },
                None => self.timer_start_utc = Some(Utc::now()),
            },
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = row![
            TaskScroller(&self.db.get_open_tasks()),
            TaskEditor(&self.draft_task, self.timer_start_utc.as_ref()),
            Calendar(&self.db.get_schedule()),
        ].into();

        container(content).into()
    }
}
