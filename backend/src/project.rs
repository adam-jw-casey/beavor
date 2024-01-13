use petgraph::{Graph, Directed, algo::kosaraju_scc};
use chrono::NaiveDate;

use crate::{Milestone, task::{Task, Start, End, BoundedTask, Id}, due_date::DueDate};

#[derive(Clone, Debug, Hash)]
enum ProjectNode {
    Milestone(Milestone),
    RawStart(NaiveDate),
    RawEnd(DueDate),
}

#[derive(Default, Debug)]
pub struct Project {
    graph: Graph<ProjectNode, Task, Directed, u32>,
}

impl Project {
    /// Creates a project from the passed tasks and milestones
    #[must_use] pub fn from_tasks_and_milestones (tasks: Box<[BoundedTask]>, milestones: Box<[Milestone]>) -> Vec<Self> {
        let mut graph = Graph::default();
        let milestone_nids: Vec<(Id, _)> =
            milestones
            .iter()
            .map(|&milestone| (milestone.get_id(), graph.add_node(ProjectNode::Milestone(milestone))))
            .collect();

        for bounded_task in tasks.iter() {
            // Get the project node that starts the task.
            // This may or may not be a milestone.
            let start_nid = match bounded_task.start {
                Start::Raw(date) => graph.add_node(ProjectNode::RawStart(date)),
                Start::Milestone(id) => *milestone_nids.iter().find_map(|(mid, nid)| {
                        if *mid == id {
                            Some(nid)
                        } else {
                            None
                        }
                    }
                ).expect("We've just inserted matching milestones. If no id matches then the database is corrupted."),
            };

            // This finds the project node that ends the task.
            // The logic here is identical to above, with the enum variants changing.
            // This could potentially be broken out into another function
            let end_nid = match bounded_task.end {
                End::Raw(due_date) => graph.add_node(ProjectNode::RawEnd(due_date)),
                End::Milestone(id) => *milestone_nids.iter().find_map(|(mid, nid)| {
                        if *mid == id {
                            Some(nid)
                        } else {
                            None
                        }
                    }
                ).expect("We've just inserted matching milestones. If no id matches then the database is corrupted."),
            };

            graph.add_edge(start_nid, end_nid, bounded_task.task);
        }

        Self::to_subgraphs(graph).iter().map(|g| Project{graph: *g}).collect()
    }

    fn to_subgraphs (g: Graph<ProjectNode, Task, Directed, u32>) -> Vec<Graph<ProjectNode, Task, Directed, u32>> {
        // At this point we have all the projects in one big, disjoint graph
        // TODO need to convert this into a Vec<Project>
        // consider petgraph::algo::scc to find the individual project graphs.
        // Some conversion will be necessary
        let components: Vec<_> = kosaraju_scc(&g);

        let subgraphs = components
            .iter()
            .map(|component: &Vec<_>| {
                let mut subg: Graph<ProjectNode, Task, Directed, u32> = Graph::new();
                
                let nids = 
                    component
                    .iter()
                    .map(|nid| 
                         subg
                         .add_node(*g.node_weight(*nid).expect("This is a valid nid"))
                    );

                

                subg
            })
            .collect();
        
        subgraphs
    }
}
