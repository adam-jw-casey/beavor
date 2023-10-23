use iced::widget::{
    Button,
    button,
    text
};

use crate::Message;

// TODO some sort of URI scheme handling would be nice here, so the user doesn't have to type it in
// manually, e.g., file://
pub fn hyperlink <'a>(url: String, display_text: String) -> Button<'a, Message>{
    button(text(display_text))
        .on_press(Message::Open(url))
}
