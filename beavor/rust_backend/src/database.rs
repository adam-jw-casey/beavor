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

use futures::future;

use sscanf::sscanf;

use itertools::Itertools;

// Helper function
fn lowest_positive_int_not_in_interator<I>(iter: I) -> u32
where I: IntoIterator<Item = u32>,
{
    let mut n = 0;
    for i in iter.into_iter().sorted(){
        if i != n{
            return n
        }

        n += 1;
    }

    n
}

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
        let default_task = Task::default();

        let available_string: String = (&default_task.available).into();
        let status_string: String = (&default_task.status).into();
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
                        DueDeliverable,
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
                        ?,
                        ?
                    )
            ",
                default_task.name,
                status_string,
                default_task.time_needed, // When creating a new task, save the initial time_needed estimate as time_budgeted
                default_task.time_needed,
                default_task.time_used,
                available_string,
                deliverable.id,
                default_task.notes,
                today_string,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert Task into database")
                .last_insert_rowid();

            sqlx::query_as::<_, Task>("
                SELECT *
                FROM tasks
                WHERE id == ?
            ")
                .bind(new_rowid)
                .fetch_one(&self.pool)
                .await
                .expect("Should have inserted and retrieved a task")
        })
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
                WHERE id      == ?
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

    fn create_external_on_deliverable(&self, deliverable: Deliverable) -> External{
        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO externals
                    (
                        Name,
                        Link,
                        Deliverable
                    )
                VALUES
                    (
                        '',
                        '',
                        ?
                    )
            ", deliverable.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to insert new external into database")
                .last_insert_rowid();

            self.get_external_by_id(new_rowid).await
        })
    }

    fn delete_external(&self, external: External){
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM externals
                WHERE id == ?
            ", external.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete external");
        });
    }

    fn update_external(&self, external: External){
        self.rt.block_on(async{
            sqlx::query!("
                UPDATE externals
                SET
                    Name  = ?,
                    Link  = ?
                WHERE id == ?
            ",
                external.name,
                external.link,
                external.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update external");
        });
    }

    fn create_deliverable_in_project(&self, project: &Project) -> Deliverable{
        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO deliverables
                    (
                        Name,
                        Project,
                        DueDate,
                        Finished,
                        Notes
                    )
                VALUES
                    (
                        '',
                        ?,
                        'None',
                        0,
                        ''
                    )
            ", project.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to insert new external into database")
                .last_insert_rowid();

            self.get_deliverable_by_id(new_rowid).await
        })
    }

    fn delete_deliverable(&self, deliverable: Deliverable){
        self.rt.block_on(async{
            // Because of CASCADE ON DELETE, this will recursively remove
            // all contained tasks and externals
            sqlx::query!("
                DELETE
                FROM deliverables
                WHERE id == ?
            ", deliverable.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete deliverable");
        });
    }

    fn update_deliverable(&self, deliverable: Deliverable){
        let due_string: String = (&deliverable.due).into();
        self.rt.block_on(async{
            sqlx::query!("
                UPDATE deliverables
                SET
                    Name    = ?,
                    DueDate = ?,
                    Notes   = ?
                WHERE id == ?
            ",
                deliverable.name,
                due_string,
                deliverable.notes,
                deliverable.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update deliverable");
        });
    }

    fn create_project_in_category(&self, category: &Category) -> Project{
        self.rt.block_on(async{
            let new_project_name = format!("Project{}", lowest_positive_int_not_in_interator(
                sqlx::query!("
                    SELECT Name
                    FROM Projects
                    WHERE
                        Name LIKE 'Project%'
                    AND Category == ?
                ", category.id)
                    .fetch_all(&self.pool)
                    .await
                    .unwrap()
                    .iter()
                    .filter_map(|cs| sscanf!(cs.Name, "Project{u32}").ok())
            ));
            println!("New project: {new_project_name}");

            let new_rowid = sqlx::query!("
                INSERT INTO projects
                    (
                        Name,
                        Category
                    )
                    VALUES
                    (
                        ?,
                        ?
                    )
            ",
                new_project_name,
                category.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to insert project into database")
                .last_insert_rowid();

            self.get_project_by_id(new_rowid).await
        })
    }

    fn delete_project(&self, project: Project){
        self.rt.block_on(async{
            // Because of CASCADE ON DELETE, this will recursively remove
            // all contained deliverables, tasks and externals
            sqlx::query!("
                DELETE
                FROM projects
                WHERE id == ?
            ", project.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete project");
        });
    }

    fn update_project(&self, project: Project){
        self.rt.block_on(async{
            sqlx::query!("
                UPDATE projects
                SET Name  = ?
                WHERE id == ?
            ",
                project.name,
                project.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update project");
        });
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

            self.get_category_by_id(new_rowid).await
        })
    }

    /// Creates a new category named "New CategoryX" where X is the lowest positive integer not yet
    /// in use
    fn create_default_category(&self) -> Category{
        let available_suffix: u32 = self.rt.block_on(async{
            lowest_positive_int_not_in_interator(
                sqlx::query!("
                    SELECT Name
                    FROM Categories
                    WHERE Name LIKE 'New Category%'
                ")
                    .fetch_all(&self.pool)
                    .await
                    .unwrap()
                    .iter()
                    .filter_map(|cs| sscanf!(cs.Name, "New Category{u32}").ok())
            )
        });

        self.create_category(format!("New Category{available_suffix}"))
    }

    fn delete_category(&self, category: Category){
        self.rt.block_on(async{
            // Because of CASCADE ON DELETE, this will recursively remove
            // all contained projects, deliverables, tasks and externals
            sqlx::query!("
                DELETE
                FROM categories
                WHERE id == ?
            ",category.id)
                .execute(&self.pool)
                .await
                .expect("Should be able to delete category");
        });
    }

    fn update_category(&self, category: Category){
        self.rt.block_on(async{
            sqlx::query!("
                UPDATE categories
                SET   Name  = ?
                WHERE id   == ?
            ",
                category.name,
                category.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update category");
        });
    }

    fn get_all(&self) -> Vec<Category>{
        self.rt.block_on(async{
            self.get_categories().await
        })
    }
}

// This disables an annoying warning from the query! macro
#[allow(non_snake_case)]
impl DatabaseManager{
    async fn get_external_by_id(&self, id: i64) -> External{
        sqlx::query_as::<_, External>("
            SELECT *
            FROM externals
            WHERE id == ?
        ")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .expect("Should have retrieved an external")
    }

    async fn get_category_by_id(&self, id: i64) -> Category{
        let cat_struct = sqlx::query!("
            SELECT *
            FROM categories
            WHERE id == ?
        ", id)
            .fetch_one(&self.pool)
            .await
            .expect("Should have retrieved a category");

        Category{
            name: cat_struct.Name.clone(),
            projects: self.get_projects_by_category_id(&cat_struct.id).await,
            id: Some(cat_struct.id),
        }
    }

    async fn get_project_by_id(&self, id: i64) -> Project{
        let proj_struct = sqlx::query!("
            SELECT *
            FROM projects
            WHERE id == ?
        ", id)
            .fetch_one(&self.pool)
            .await
            .expect("Should have retrieved a project");

        Project{
            name: proj_struct.Name.clone(),
            deliverables: self.get_deliverables_by_project_id(&proj_struct.id).await,
            id: Some(proj_struct.id),
        }
    }

    async fn get_deliverable_by_id(&self, id: i64) -> Deliverable{
        let deliv_struct = sqlx::query!("
            SELECT *
            FROM deliverables
            WHERE id == ?
        ", id)
            .fetch_one(&self.pool)
            .await
            .expect("Should have retrieved a deliverable");

        Deliverable{
            name:      deliv_struct.Name.clone(),
            due:   (&deliv_struct.DueDate).try_into().expect("Should be well-formatted"),
            notes:     deliv_struct.Notes.clone(),
            tasks:     self.get_tasks_by_deliverable_id(&deliv_struct.id).await,
            externals: self.get_externals_by_deliverable_id(&deliv_struct.id).await,
            id:        Some(deliv_struct.id),
        }
    }

    async fn get_categories(&self) -> Vec<Category>{
        future::join_all(sqlx::query!("
            SELECT id
            FROM categories
        ")
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .iter()
            .map(|cs| async{self.get_category_by_id(cs.id).await}))
            .await
    }

    async fn get_projects_by_category_id(&self, id: &i64) -> Vec<Project>{
        future::join_all(sqlx::query!("
            SELECT id
            FROM projects
            WHERE Category == ?
        ", id)
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .iter()
            .map(|ps| async{self.get_project_by_id(ps.id).await}))
            .await
    }

    async fn get_deliverables_by_project_id(&self, id: &i64) -> Vec<Deliverable>{
        future::join_all(sqlx::query!("
            SELECT *
            FROM deliverables
            WHERE Project == ?
        ", id)
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .iter()
            .map(|ds| async{self.get_deliverable_by_id(ds.id).await}))
            .await
    }

    async fn get_tasks_by_deliverable_id(&self, id: &i64) -> Vec<Task>{
        sqlx::query("
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
            .collect()
    }

    async fn get_externals_by_deliverable_id(&self, id: &i64) -> Vec<External>{
        sqlx::query("
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
            .collect()
    }
}

impl FromRow<'_, SqliteRow> for External{
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(External{
            name:    row.try_get("Name")?,
            link:    row.try_get("Link")?,
            id: Some(row.try_get("id")?),
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
            id:     Some(row.try_get("id")?),
        })
    }
}
