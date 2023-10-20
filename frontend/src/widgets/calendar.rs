use iced::widget::{
    Column,
    row,
    Row,
    column,
    button,
    text,
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

use crate::Message;

#[derive(Debug, Clone, Default)]
pub struct CalendarState{
    weeks_scrolled: u16,
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
                        .map(|d| Element::from(cal_day(d, schedule.workload_on_day(d)).padding(4)))
                        .collect()
                    ).into()
                ).collect()
        )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(8),
            column![
                button("↑").on_press_maybe(if state.weeks_scrolled > 0 {Some(Message::ScrollUpMaxCalendar)}else{None}),
                button("🞁").on_press_maybe(if state.weeks_scrolled > 0 {Some(Message::ScrollUpCalendar)}else{None}),
                button("🞃").on_press(Message::ScrollDownCalendar),
            ]
    ].into()
}

fn cal_day(day: NaiveDate, load: Option<u32>) -> Column<'static, Message>{
    column![
        text(
            day.format("%b %d")
        ),
        text(
            if let Some(load) = load {format!("{:.1}", f64::from(load)/60.0)} else{"-".to_string()}
        )
    ]
        .padding(4)
        .align_items(Alignment::Center)
}
