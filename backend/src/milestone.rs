use crate::due_date::DueDate;
use crate::task::Id;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Milestone {
    pub name:       String,
    pub category:   String,
    pub due_date:   DueDate,
    pub id:         Id,
}
