use iced::widget::{
    Column,
    container,
    Row,
    row,
    column,
    scrollable,
    text,
    Container,
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
    Schedule,
};

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
enum Message{
    SelectTask(Task),
    DeselectTask,
    SelectDate(NaiveDate),
    DeselectDate,
}

struct Beavor{
    db: DatabaseManager,
    selected_task: Option<Task>,
    selected_date: Option<NaiveDate>,
}

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        let db = DatabaseManager::new("worklist.db".into());
        Self{
            db,
            selected_task: None,
            selected_date: None,
        }
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
        match message{
            Message::SelectTask(task) => self.selected_task = Some(task),
            Message::DeselectTask     => self.selected_task = None,
            Message::SelectDate(date) => self.selected_date = Some(date),
            Message::DeselectDate     => self.selected_date = None,
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = row![
            TaskScroller(&self.db.get_open_tasks()),
            Calendar(&self.db.get_schedule()),
        ].into();

        container(content).into()
    }
}

#[allow(non_snake_case)]
fn Calendar(schedule: &Schedule) -> Element<'static, Message>{
    // TODO this is a pretty grungy implementation, but it should do for now
    // Get the days of the week that contains the passed day
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
    // Show this week and the following ones
    Column::with_children(
        [
        today,
        today + Days::new(7),
        today + Days::new(14),
        today + Days::new(21),
        ]
            .iter()
            .map(|d| Row::with_children(
                week_of(*d).iter().map(
                    |d| Element::from(CalDay(*d, schedule.workload_on_day(*d)).padding(4))
                ).collect()
            ).into())
            .collect()
    ).into()
}

#[allow(non_snake_case)]
fn CalDay(day: NaiveDate, load: u32) -> Column<'static, Message>{
    column![
        text(
            day.format("%b %d")
        ),
        text(
            format!("{}", load as f32/60.0)
        )
    ]
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
        .on_press(Message::SelectTask(task.clone()))
        .into()
}
