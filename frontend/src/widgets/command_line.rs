use iced::widget::{
    column,
    Column,
    text_input,
    text,
};

use crate::Message;

#[derive(Debug, Clone, Default)]
pub struct State{
    pub command: String,
    pub error: Option<String>,
}

pub fn command_line(state: &State) -> Column<'_, Message>{
    column![
        text_input("", &state.command)
            .on_input(Message::UpdateCommandLine)
            .on_submit(Message::RunCommand),
        text(state.error.clone().unwrap_or_default()),
    ]
}
