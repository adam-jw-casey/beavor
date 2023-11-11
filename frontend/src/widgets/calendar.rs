use iced::widget::{
    Column,
    row,
    Row,
    column,
    button,
    text,
    MouseArea,
};

use iced::{
    Element,
    Length,
    Alignment,
};

use chrono::{
    Weekday,
    NaiveDate,
    naive::Days,
};

use backend::{
    utils::today_date,
    Schedule,
};

use iced_aw::{
    Icon,
    graphics::icons::{
        icon_to_char,
        ICON_FONT,
    },
};

use crate::Message;

#[derive(Debug, Clone, Default)]
pub struct CalendarState{
    weeks_scrolled: u16,
    pub clicked_date: Option<NaiveDate>,
    pub filter_date: Option<NaiveDate>,
}

impl CalendarState{
    pub fn scroll_down(&mut self){
        self.weeks_scrolled += 1;
    }

    pub fn scroll_up(&mut self){
        self.weeks_scrolled = self.weeks_scrolled.saturating_sub(1);
    }

    pub fn scroll_up_max(&mut self) {
        self.weeks_scrolled = 0;
    }
}

// TODO this needs day-of-week headers
pub fn calendar(schedule: &Schedule, state: &CalendarState) -> Element<'static, Message>{
    // Get the days of the week that contains the passed day
    fn week_of(d: NaiveDate) -> Vec<NaiveDate>{
        let w = d.week(Weekday::Mon);

        // Monday to Friday
        (0..=4)
            .map(|n| w.first_day() + Days::new(n))
            .collect()
    }

    let today = today_date();

    let num_weeks = 4;
    row![
        Row::with_children(
            week_of(today + Days::new((7*state.weeks_scrolled).into()))
                .iter()
                .map(|d| Column::with_children(
                    (0..num_weeks)
                        .map(|n| *d + Days::new(7*n))
                        .map(|d| Element::from(cal_day(d, schedule.workload_on_day(d), Some(d) == state.filter_date, state.clicked_date.as_ref())))
                        .collect()
                    ).into()
                ).collect()
        )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8),
            column![
                button(text(icon_to_char(Icon::ChevronDoubleUp)).font(ICON_FONT)).on_press_maybe(if state.weeks_scrolled > 0 {Some(Message::ScrollUpMaxCalendar)}else{None}),
                button(text(icon_to_char(Icon::ChevronUp)).font(ICON_FONT)).on_press_maybe(if state.weeks_scrolled > 0 {Some(Message::ScrollUpCalendar)}else{None}),
                button(text(icon_to_char(Icon::ChevronDown)).font(ICON_FONT)).on_press(Message::ScrollDownCalendar),
            ].height(Length::Shrink)
    ]
        .align_items(Alignment::Center)
        .into()
}

fn cal_day(day: NaiveDate, load: Option<u32>, is_selected: bool, clicked_date: Option<&NaiveDate>) -> Element<'static, Message>{
    MouseArea::new(
        column![
            text(
                if is_selected{
                    day.format("[%b %d]")
                }else{
                    day.format("%b %d")
                }
            ),
            text(
                if let Some(load) = load {format!("{:.1}", f64::from(load)/60.0)} else{"-".to_string()}
            )
        ]
            .padding(4)
            .align_items(Alignment::Center)
    )
        .on_press(Message::ClickDate(Some(day)))
        .on_release(
            if clicked_date.is_some_and(|d| *d == day){
                Message::FilterToDate(Some(day))
            }else{
                Message::ClickDate(None)
        })
        .into()
}
