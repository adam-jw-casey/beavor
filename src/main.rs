use iced::widget::{
    column,
    container,
    row,
    scrollable,
    text,
};

use iced::{
    Sandbox,
    Element,
    Settings,
};

fn main() {
    Beavor::run(Settings::default())
        .expect("Application failed");
}

#[derive(Debug, Clone)]
enum Message{
}

struct Beavor;

impl Sandbox for Beavor {
    type Message = Message;

    fn new() -> Self {
        Self{}
    }

    fn title(&self) -> String {
        String::from("Beavor")
    }

    fn update(&mut self, message: Self::Message) {
        match message{
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let task_scroller: Element<Message> =
            scrollable(
                column![
                    text("hello"),
                    text("world"),
                ]
            ).into();

        let content: Element<Message> = row![task_scroller].into();

        container(content).into()
    }
}
