use pyo3::prelude::{
    pyclass,
    pymethods,
    PyResult,
};
use pyo3::types::PyType;

use tokio::runtime::Runtime;

use sqlx::sqlite::{
    SqlitePool,
    SqlitePoolOptions,
    SqliteConnectOptions,
    SqliteRow,
};
use sqlx::{
    Executor,
    ConnectOptions,
    FromRow,
    Row,
};

use std::str::FromStr;

use crate::{
    Category,
    Project,
    External,
    Task,
    Deliverable,
    today_str,
};

#[pyclass]
pub struct DatabaseManager{
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

    fn create_task_on_deliverable(&self, deliverable: Deliverable) -> Task{
        let mut new_task = Task::new(&deliverable);

        let available_string: String = (&new_task.available).into();
        let status_string: String = (&new_task.status).into();
        let today_string = today_str();

        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO tasks
                    (
                        Name,
                        Status,
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
                new_task.name,
                status_string,
                new_task.time_needed, // When creating a new task, save the initial time_needed estimate as time_budgeted
                new_task.time_needed,
                new_task.time_used,
                available_string,
                new_task.notes,
                today_string,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert Task into database")
                .last_insert_rowid();

            new_task = sqlx::query_as::<_, Task>("
                SELECT *
                FROM tasks
                WHERE TaskID == ?
            ")
                .bind(new_rowid)
                .fetch_one(&self.pool)
                .await
                .expect("Should have inserted and retrieved a task");
        });

        new_task
    }

    fn delete_task(&self, task: Task){
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM tasks
                WHERE rowid == ?
            ", task.id)
                .execute(&self.pool)
                .await
                .expect("Should be able do delete task");
        });
    }

    fn update_task(&self, task: Task){
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let available_string: String = (&task.available).into();
        let status_string: String = (&task.status).into();

        self.rt.block_on(async{
            sqlx::query!("
                UPDATE tasks
                SET
                    Name       = ?,
                    Status     = ?,
                    TimeNeeded = ?,
                    TimeUsed   = ?,
                    Available  = ?,
                    Notes      = ?
                WHERE
                    TaskID    == ?
            ",
                task.name,
                status_string,
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
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM externals
                WHERE ExternalID == ?
            ", external.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete external");
        });
    }

    fn update_external(&self, external: External){
        todo!();
    }

    fn create_deliverable_in_project(&self, project: &Project) -> Deliverable{
        todo!();
    }

    fn delete_deliverable(&self, deliverable: Deliverable){
        self.rt.block_on(async{
            // Because of CASCADE ON DELETE, this will recursively remove
            // all contained tasks and externals
            sqlx::query!("
                DELETE
                FROM deliverables
                WHERE DeliverableID == ?
            ", deliverable.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete deliverable");
        });
    }

    fn update_deliveral(&self, deliverable: Deliverable){
        todo!();
    }

    fn create_project_in_category(&self, category: &Category) -> Project{
        todo!();
    }

    fn delete_project(&self, project: Project){
        self.rt.block_on(async{
            // Because of CASCADE ON DELETE, this will recursively remove
            // all contained deliverables, tasks and externals
            sqlx::query!("
                DELETE
                FROM projects
                WHERE ProjectID == ?
            ", project.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete project");
        });
    }

    fn update_project(&self, project: Project){
        todo!();
    }

    fn create_category(&self, name: String) -> Category{
        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO categories
                    (Name)
                VALUES
                    (?)
            ",
                name,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert category into database")
                .last_insert_rowid();

            let cat_struct = sqlx::query!("
                SELECT *
                FROM Categories
                WHERE CategoryID == ?
            ", new_rowid)
                .fetch_one(&self.pool)
                .await
                .expect("Should have inserted and retrieved a category");

            Category{
                name: cat_struct.Name,
                projects: Vec::new(),
                id: Some(cat_struct.CategoryID),
            }
        })
    }

    fn delete_category(&self, category: Category){
        self.rt.block_on(async{
            // Because of CASCADE ON DELETE, this will recursively remove
            // all contained projects, deliverables, tasks and externals
            sqlx::query!("
                DELETE
                FROM categories
                WHERE CategoryID == ?
            ",category.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete category");
        });
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
                    name: cs.Name.clone(),
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
                    name: ps.Name.clone(),
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
                    name: ds.Name.clone(),
                    due: (&ds.DueDate).try_into().expect("Should be well-formatted"),
                    notes: ds.Notes.clone(),
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
            tasks = sqlx::query("
                SELECT *
                FROM tasks
                WHERE DueDeliverable == ?
            ")
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|tr| Task::from_row(tr).expect("Should produce valid tasks"))
                .collect();
        });

        tasks
    }

    fn get_externals_by_deliverable_id(&self, id: &i64) -> Vec<External>{
        let mut externals: Vec<External> = Vec::new();

        self.rt.block_on(async{
            externals = sqlx::query("
                SELECT *
                FROM externals
                WHERE Deliverable == ?
            ")
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .unwrap()
                .iter()
                .map(|es| External::from_row(es).expect("Should produce valid externals"))
                .collect();
        });

        externals
    }
}

impl FromRow<'_, SqliteRow> for External{
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(External{
            name:    row.try_get("Name")?,
            link:    row.try_get("Link")?,
            id: Some(row.try_get("ExternalID")?),
        })
    }
}

impl FromRow<'_, SqliteRow> for Task{
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self{
            name:        row.try_get("Name")?,
            status:    (&row.try_get::<String, &str>("Status")?).try_into().unwrap(),
            time_needed: row.try_get("TimeNeeded")?,
            time_used:   row.try_get("TimeUsed")?,
            available: (&row.try_get::<String, &str>("Available")?).try_into().unwrap(),
            notes:       row.try_get("Notes")?,
            id:     Some(row.try_get("TaskID")?),
        })
    }
}
