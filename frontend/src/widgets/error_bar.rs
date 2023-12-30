use iced::widget::{
    column,
    Column,
    text,
};

use crate::Message;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub error: Option<String>,
}

pub fn error_bar(state: &State) -> Column<'_, Message> {
    column![ text(state.error.clone().unwrap_or_default()) ]
}
