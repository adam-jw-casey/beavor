use iced::widget::{
    button,
    text,
    row,
    text_input,
};

use iced::Element;

use crate::Message;

use backend::Hyperlink;

// TODO some sort of URI scheme handling would be nice here, so the user doesn't have to type it in
// manually, e.g., file://
pub fn hyperlink <'a>(links: Vec<Hyperlink>, link_id: usize, editing_id: Option<usize>) -> Element<'a, Message>{

    let idx = links.iter().position(|l| l.id == link_id).unwrap();

    if editing_id.is_some_and(|e_id| e_id == link_id){
        row![
            text_input("name...", &links[idx].display),
            text_input("url...", &links[idx].url),
            button(text("Close"))
                .on_press(Message::EditLinkID(None)),
        ].into()
    }else{
        row![
            button(text(&links[idx].display))
                .on_press(Message::Open(links[idx].url.to_string())),
            button(text("Edit"))
                .on_press(Message::EditLinkID(Some(links[idx].id))),
        ]
            .into()
    }
}
