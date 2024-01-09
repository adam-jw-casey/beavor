use crate::{Task, Milestone};
use petgraph::{Graph, Directed};

#[derive(Default, Debug)]
pub struct Project {
    graph: Graph<Milestone, Task, Directed, u8>,
}

impl Project {
    #[must_use] pub fn new (tasks: Box<[Task]>, milestones: Box<[Milestone]>) -> Self {
        todo!()
    }
}
