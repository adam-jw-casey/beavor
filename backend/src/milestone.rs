use chrono::NaiveDate;

use crate::due_date::DueDate;
use crate::task::Id;

use crate::utils::format_date_borrowed;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Milestone {
    // TODO I don't like that this duplicates the data of task.next_action_date and task.due_date
    AnonymousStart(NaiveDate),
    AnonymousEnd(DueDate),
    Concrete {
        name:     String,
        category: String,
        due_date: DueDate,
        id:       Id,
    },
}

impl Milestone {
    #[must_use] fn get_name(&self) -> String {
        match self {
            Milestone::AnonymousStart(date) => format_date_borrowed(date),
            Milestone::AnonymousEnd(due_date) => due_date.into(),
            Milestone::Concrete { name, category: _, due_date: _, id: _ } => name.into(),
        }
    }
}
