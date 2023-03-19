use pyo3::prelude::{
    pyfunction,
    pymodule,
    pyclass,
    pymethods,
    PyResult,
    PyModule,
    Python
};
use pyo3::types::PyType;
use pyo3::wrap_pyfunction;

use tokio::runtime::Runtime;

use sqlx::sqlite::{
    SqlitePool,
    SqlitePoolOptions,
    SqliteRow,
    SqliteConnectOptions,
};
use sqlx::{
    Executor,
    ConnectOptions,
    Row,
};

use std::str::FromStr;

mod date;
use date::{
    PyDueDate,
    PyDueDateType,
    ParseDateError,
    PyAvailability,
    PyAvailabilityType,
    today_date,
    parse_date,
    format_date,
    today_str,
};

mod model;
use model::{
    Task,
    Deliverable,
    Project,
    Category,
    External,
};

#[pyfunction]
fn green_red_scale(low: f32, high: f32, val: f32) -> String {
    let frac = f32::max(0.0,f32::min(1.0,(val-low)/(high-low)));

    let red: u8;
    let green: u8;

    if frac > 0.5{
        red = 255;
        green = ((2.0-2.0*frac) * 255.0) as u8;
    }else{
        red = ((2.0*frac) * 255.0) as u8;
        green = 255
    }

    format!("#{red:02X}{green:02X}00")
}

#[pyclass]
struct DatabaseManager{
    pool: SqlitePool,
    rt: Runtime,
}

// TODO should make all these pass the asyncness through to Python to deal with
#[pymethods]
impl DatabaseManager{
    #[new]
    fn new(database_path: String) -> PyResult<Self>{
        let rt = Runtime::new().unwrap();
        Ok(Self{
            pool: rt.block_on(
                      SqlitePoolOptions::new()
                      .after_connect(|conn, _meta| Box::pin( async {
                          conn.execute("PRAGMA foreign_keys=ON").await?;
                        Ok(())
                      }))
                      .connect(database_path.as_str())
                    ).expect("Should be able to connect to database"),
            rt,
        })
    }

    #[classmethod]
    fn create_new_database(_cls: &PyType, database_path: String){
        let rt = Runtime::new().unwrap();
        rt.block_on(async{
            let mut conn = SqliteConnectOptions::from_str(&database_path)
                .expect("This should work")
                .create_if_missing(true)
                .connect()
                .await
                .expect("Should be able to connect to database");

            sqlx::query_file!("resources/schema.sql")
                .execute(&mut conn)
                .await
                .expect("Should be able to create the schema");
        });
    }

    // TODO update this to use new schema properly
    fn create_task_on_deliverable(&self, deliverable: Deliverable) -> Task{
        todo!();
        let mut new_task = Task::default();

        let available_string: String = (&task.available).into();
        let today_string = today_str();

        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO tasks
                    (
                        Name,
                        Finished,
                        TimeBudgeted,
                        TimeNeeded,
                        TimeUsed,
                        Available,
                        Notes,
                        DateAdded
                    )
                VALUES
                    (
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?
                    )
            ",
                task.name,
                task.finished,
                task.time_needed, // When creating a new task, save the initial time_needed estimate as time_budgeted
                task.time_needed,
                task.time_used,
                available_string,
                task.notes,
                today_string,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert Task into database")
                .last_insert_rowid();

            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            new_task = sqlx::query("
                SELECT *, rowid
                FROM tasks
                WHERE rowid == ?
            ")
                .bind(new_rowid)
                .fetch_one(&self.pool)
                .await
                .expect("Should have inserted and retrieved a task")
                .try_into()
                .expect("Database should contain valid Tasks only");
        });

        new_task
    }

    fn delete_task(&self, task: Task){
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM tasks
                WHERE rowid == ?
            ",
                task.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able do delete task");
        });
    }

    // TODO update this to use new schema properly
    fn update_task(&self, task: Task){
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let available_string: String = (&task.available).into();

        self.rt.block_on(async{
            sqlx::query!("
                UPDATE tasks
                SET
                    Finished =    ?,
                    Name =        ?,
                    TimeNeeded =  ?,
                    TimeUsed =    ?,
                    Available =   ?,
                    Notes =       ?
                WHERE
                    rowid ==      ?
            ",
                task.finished,
                task.name,
                task.time_needed,
                task.time_used,
                available_string,
                task.notes,
                task.id,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update task");
        })
    }


    fn create_external_on_deliverable(&self, deliverable: Deliverable) -> Task{
        todo!();
    }

    fn delete_external(&self, external: External){
        todo!();
    }

    fn update_external(&self, external: External){
        todo!();
    }

    fn create_deliverable_in_project(&self, project: &Project) -> Deliverable{
        todo!();
    }

    fn delete_deliverable(&self, deliverable: Deliverable){
        todo!();
    }

    fn update_deliveral(&self, deliverable: Deliverable){
        todo!();
    }

    fn create_project_in_category(&self, category: &Category) -> Project{
        todo!();
    }

    fn delete_project(&self, project: Project){
        todo!();
    }

    fn update_project(&self, project: Project){
        todo!();
    }

    fn create_category(&self, name: String) -> Category{
        todo!();
    }

    fn delete_category(&self, category: Category){
        todo!();
    }

    fn update_category(&self, category: Category){
        todo!();
    }

    fn get_all(&self) -> Vec<Category>{
        self.get_categories()
    }
}

impl DatabaseManager{
    fn get_categories(&self) -> Vec<Category>{
        let mut categories: Vec<Category> = Vec::new();

        self.rt.block_on(async{
            categories = sqlx::query!("
                SELECT *
                FROM categories
            ")
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|cs| Category{
                    name: cs.Name,
                    projects: self.get_projects_by_category_id(&cs.CategoryID),
                    id: Some(cs.CategoryID),
                })
                .collect();
        });

        categories
    }

    fn get_projects_by_category_id(&self, id: &i64) -> Vec<Project>{
        let mut projects: Vec<Project> = Vec::new();

        self.rt.block_on(async{
            projects = sqlx::query!("
                SELECT *
                FROM projects
                WHERE Category == ?
            ", id)
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|ps| Project{
                    name: ps.Name,
                    deliverables: self.get_deliverables_by_project_id(&ps.ProjectID),
                    id: Some(ps.ProjectID),
                })
                .collect();
        });

        projects
    }

    fn get_deliverables_by_project_id(&self, id: &i64) -> Vec<Deliverable>{
        let mut deliverables: Vec<Deliverable> = Vec::new();

        self.rt.block_on(async{
            deliverables = sqlx::query!("
                SELECT *
                FROM deliverables
                WHERE Project == ?
            ", id)
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|ds| Deliverable{
                    name: ds.Name,
                    due: ds.DueDate.try_into().expect("Should be well-formatted"),
                    notes: ds.Notes,
                    tasks: self.get_tasks_by_deliverable_id(&ds.DeliverableID),
                    externals: self.get_externals_by_deliverable_id(&ds.DeliverableID),
                    id: Some(ds.DeliverableID),
                })
                .collect();
        });

        deliverables
    }

    fn get_tasks_by_deliverable_id(&self, id: &i64) -> Vec<Task>{
        let mut tasks: Vec<Task> = Vec::new();

        self.rt.block_on(async{
            tasks = sqlx::query!("
                SELECT *
                FROM tasks
                WHERE DueDeliverable == ?
            ", id)
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|ts| Task{
                    name: ts.Name,
                    status: (&ts.Status).try_into().expect("Should be formatted correctly"),
                    time_needed: ts.TimeNeeded as i32,
                    time_used: ts.TimeUsed as i32,
                    available: ts.Available.try_into().expect("Should be formatted correctly"),
                    notes: ts.Notes,
                    id: Some(ts.TaskID),
                })
                .collect();
        });

        tasks
    }

    fn get_externals_by_deliverable_id(&self, id: &i64) -> Vec<External>{
        let mut externals: Vec<External> = Vec::new();

        self.rt.block_on(async{
            externals = sqlx::query!("
                SELECT *
                FROM externals
                WHERE Deliverable == ?
            ", id)
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|es| External{
                    name: es.Name,
                    link: es.Link,
                    id: Some(es.ExternalID),
                })
                .collect();
        });

        externals
    }
}

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_date, m)?)?;
    m.add_function(wrap_pyfunction!(green_red_scale, m)?)?;
    m.add_function(wrap_pyfunction!(parse_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_date, m)?)?;
    m.add_function(wrap_pyfunction!(today_str, m)?)?;
    m.add_class::<Task>().unwrap();
    m.add_class::<Deliverable>().unwrap();
    m.add_class::<Project>().unwrap();
    m.add_class::<Category>().unwrap();
    m.add_class::<External>().unwrap();
    m.add_class::<PyDueDate>().unwrap();
    m.add_class::<PyDueDateType>().unwrap();
    m.add_class::<DatabaseManager>().unwrap();
    m.add_class::<PyAvailability>().unwrap();
    m.add_class::<PyAvailabilityType>().unwrap();
    Ok(())
}
