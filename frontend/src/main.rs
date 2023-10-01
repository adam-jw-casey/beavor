use iced::widget::{
    Column,
    container,
    Row,
    row,
    column,
    scrollable,
    text,
    Button,
};

use iced::{
    Sandbox,
    Element,
    Settings,
    Length,
};

use chrono::{
    Weekday,
    NaiveDate,
    naive::Days,
};

use backend::{
    DatabaseManager,
    Task,
    utils::today_date,
};

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
enum Message{
    SelectTask(Option<Task>),
}

struct Beavor{
    db: DatabaseManager,
    open_tasks: Vec<Task>,
    selected_task: Option<Task>,
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        let db = DatabaseManager::new("worklist.db".into());
        Self{
            open_tasks: db.get_open_tasks(),
            db,
            selected_task: None,
        }
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
        match message{
            Message::SelectTask(task) => self.selected_task = task,
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = row![
            TaskScroller(&self.open_tasks),
            Calendar(&self.open_tasks),
        ].into();

        container(content).into()
    }
}

#[allow(non_snake_case)]
fn Calendar(tasks: &[Task]) -> Element<'static, Message>{
    fn week_of(d: NaiveDate) -> Vec<NaiveDate>{
        let w = d.week(Weekday::Mon);

        vec![
            w.first_day(),
            w.first_day() + Days::new(1),
            w.first_day() + Days::new(2),
            w.first_day() + Days::new(3),
            w.first_day() + Days::new(4),
        ]
    }

    let today = today_date();
    

    // TODO this is a pretty grungy implementation, but it should do for now
    Column::with_children(
        [
        today,
        today + Days::new(7),
        today + Days::new(14),
        today + Days::new(21),
        ]
            .iter()
            .map(|d| Row::with_children(
                week_of(*d).iter().map(|d| Element::into(CalDay(*d))).collect()
            ).into())
            .collect()
    ).into()
}

#[allow(non_snake_case)]
fn CalDay(d: NaiveDate) -> Element<'static, Message>{
    text(d).into()
}

#[allow(non_snake_case)]
fn TaskScroller(tasks: &[Task]) -> Element<'static, Message>{
    scrollable(
        Column::with_children(
            tasks
                .iter()
                .map(TaskRow)
                .collect()
        )
            .width(Length::Shrink)
            .padding([40, 0, 40, 0])
    ).into()
}

#[allow(non_snake_case)]
fn TaskRow(task: &Task) -> Element<'static, Message>{
    Button::new(
        text(&task.name)
    )
        .on_press(Message::SelectTask(Some(task.clone())))
        .into()
}
