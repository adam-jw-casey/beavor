use iced::widget::{
    button,
    text,
    row,
    text_input,
};

use iced::Element;

use crate::Message;
use crate::widgets::task_editor::{UpdateDraftTask, LinkMessage};

use backend::Hyperlink;

// TODO some sort of URI scheme handling would be nice here, so the user doesn't have to type it in
// manually, e.g., file://
pub fn hyperlink <'a>(links: &[Hyperlink], link_id: usize, editing_id: Option<usize>) -> Element<'a, Message>{

    let idx = links.iter().position(|l| l.id == link_id).unwrap();
    let link = links[idx].clone();

    // This is janky but it works?
    let display_link = link.clone();
    let url_link = link.clone();

    if editing_id.is_some_and(|e_id| e_id == link_id){
        row![
            text_input("name...", &link.display).on_input(move |s: String| {
                let mut new_link = display_link.clone();
                new_link.display = s;
                Message::UpdateDraftTask(UpdateDraftTask::Link(LinkMessage::Update(new_link))
            )}),
            text_input("url...", &link.url).on_input(move |s: String| {
                let mut new_link = url_link.clone();
                new_link.url = s;
                Message::UpdateDraftTask(UpdateDraftTask::Link(LinkMessage::Update(new_link))
            )}),
            button(text("Close"))
                .on_press(Message::EditLinkID(None)),
        ]
            .spacing(4)
            .into()
    }else{
        row![
            button(text(&links[idx].display))
                .on_press(Message::Open(link.url.to_string())),
            button(text("Edit"))
                .on_press(Message::EditLinkID(Some(link.id))),
            button(text("Delete"))
                .on_press(Message::UpdateDraftTask(UpdateDraftTask::Link(LinkMessage::Delete(link_id)))),
        ]
            .spacing(4)
            .into()
    }
}
