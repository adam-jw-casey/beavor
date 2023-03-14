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
use pyo3::{
    wrap_pyfunction,
    PyErr,
};
use pyo3::exceptions::{
    PyValueError,
    PyNotImplementedError,
};
use pyo3::basic::CompareOp;

use tokio::runtime::Runtime;

use sqlx::sqlite::{
    SqlitePool,
    SqliteRow,
    SqliteConnectOptions,
};
use sqlx::{
    Row,
    ConnectOptions
};

use chrono::naive::NaiveDate;
use chrono::Local;

use std::str::FromStr;
use std::convert::From;
use core::fmt::Display;

#[pyfunction]
// Tested and this is ~3x faster than the exact same implementation in Python,
// even with the API calls
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

#[pyfunction]
fn format_date(date: NaiveDate) -> String{
    format_date_borrowed(&date)
}

fn format_date_borrowed(date: &NaiveDate) -> String{
    date.format("%F").to_string()
}

#[pyfunction]
fn parse_date(date_string: &str) -> Result<NaiveDate, ParseDateError>{
    match NaiveDate::parse_from_str(date_string, "%F"){
        Ok(nd) => Ok(nd),
        _ => Err(ParseDateError)
    }
}

#[pyfunction]
fn today_str() -> String{
    format_date(today_date())
}

#[pyfunction]
fn today_date() -> NaiveDate{
    Local::now().naive_local().date()
}

fn work_days_between(d1: NaiveDate, d2: NaiveDate) -> i32{
    todo!();
}

#[pyclass]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, PartialEq)]
enum PyDueDateType{
    None,
    Date,
    ASAP,
}

#[pyclass]
#[derive(Clone, PartialEq)]
struct PyDueDate{
    #[pyo3(get, set)]
    date_type: PyDueDateType,
    #[pyo3(get, set)]
    date: Option<NaiveDate>,
}

#[pymethods]
impl PyDueDate{
    fn __str__(&self) -> String{
        (&DueDate::from(self)).into()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op{
            CompareOp::Eq => Ok(*self == *other),
            CompareOp::Ne => Ok(*self != *other),
            _ => Err(PyNotImplementedError::new_err(format!("{:#?} is not implemented for PyDueDate", op))),
        }
    }

    #[classmethod]
    fn parse(_cls: &PyType, s: String) -> PyResult<Self>{
        Ok(DueDate::try_from(s)?.into())
    }
}

#[derive(Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum DueDate{
    None,
    Date(NaiveDate),
    ASAP,
}

impl From<DueDate> for PyDueDate{
    fn from(rust_due_date: DueDate) -> Self {
        match rust_due_date{
            DueDate::None => PyDueDate{date_type: PyDueDateType::None, date: None},
            DueDate::Date(date) => PyDueDate{date_type: PyDueDateType::Date, date: Some(date)},
            DueDate::ASAP => PyDueDate{date_type: PyDueDateType::ASAP, date: None},
        }
    }
}

impl From<&PyDueDate> for DueDate{
    fn from(pyvalue: &PyDueDate) -> Self {
        match pyvalue.date_type{
            PyDueDateType::None => DueDate::None,
            PyDueDateType::Date => DueDate::Date(pyvalue.date.expect("If PyDueDateType is Date then date will no be None")),
            PyDueDateType::ASAP => DueDate::ASAP,
        }
    }
}

#[derive(Debug)]
struct ParseDateError;

impl From<ParseDateError> for PyErr{
    fn from(_: ParseDateError) -> Self {
        PyValueError::new_err("Error parsing date")
    }
}

impl FromStr for DueDate{
    type Err = ParseDateError;

    fn from_str(date_string: &str) -> Result<Self, Self::Err> {
        Ok(match date_string{
            "None" => DueDate::None,
            "ASAP" => DueDate::ASAP,
            date_string => DueDate::Date(parse_date(date_string)?),
        })
    }
}

impl TryFrom<String> for DueDate{
    type Error = ParseDateError;

    fn try_from(date_string: String) -> Result<Self, Self::Error> {
        DueDate::from_str(&date_string)
    }
}

impl From<&DueDate> for String{
    fn from(value: &DueDate) -> Self {
        match value{
            DueDate::None => "None".into(),
            DueDate::ASAP => "ASAP".into(),
            DueDate::Date(date) => format_date_borrowed(date),
        }
    }
}

impl Display for DueDate{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Clone)]
#[pyclass]
struct Task{
    #[pyo3(get, set)]
    category:         String,
    #[pyo3(get, set)]
    finished:         String,
    #[pyo3(get, set)]
    task_name:        String,
    #[pyo3(get)]
    _time_budgeted:   i32,
    #[pyo3(get, set)]
    time_needed:      i32,
    #[pyo3(get, set)]
    time_used:        i32,
    #[pyo3(get, set)]
    next_action_date: NaiveDate,
    #[pyo3(get, set)]
    notes:            String,
    #[pyo3(get, set)]
    date_added:       NaiveDate,
    #[pyo3(get)]
    id:               Option<i32>,
    due_date:         DueDate,
}

#[pymethods]
impl Task{
    #[getter]
    fn get_due_date(&self) -> PyDueDate{
        self.due_date.into()
    }

    #[setter]
    fn set_due_date(&mut self, due_date: PyDueDate){
        self.due_date = (&due_date).into()
    }
}

impl TryFrom<SqliteRow> for Task{
    type Error = ParseDateError;

    fn try_from(row: SqliteRow) -> Result<Self, Self::Error> {
        Ok(Task{
            category:                     row.get::<String, &str>("Category"),
            finished:                     row.get::<String, &str>("O"),
            task_name:                    row.get::<String, &str>("Task"),
            _time_budgeted:               row.get::<i32,    &str>("Budget"),
            time_needed:                  row.get::<i32,    &str>("Time"),
            time_used:                    row.get::<i32,    &str>("Used"),
            next_action_date: parse_date(&row.get::<String, &str>("NextAction"))?,
            due_date:                     row.get::<String, &str>("DueDate").try_into()?,
            notes:                        row.get::<String, &str>("Notes"),
            id:                           row.get::<Option<i32>, &str>("rowid"),
            date_added:       parse_date(&row.get::<String, &str>("DateAdded"))?,
        })
    }
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
            pool: rt.block_on(SqlitePool::connect(database_path.as_str()))
                .expect("Should be able to connect to database"),
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

            // This doesn't use query! because when creating a database, it doesn't make sense to
            // check against an existing database
            sqlx::query_file!("resources/schema.sql")
                .execute(&mut conn)
                .await
                .expect("Should be able to create the schema");
        });
    }

    fn create_task(&self, task: Task) -> Task{
        let mut new_task = self.default_task();

        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let due_date_str = task.due_date.to_string();
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let date_added_str = DueDate::Date(task.date_added).to_string();

        self.rt.block_on(async{
            let new_rowid: i64 = sqlx::query!("
                INSERT INTO worklist
                    (
                        Category,
                        O,
                        Task,
                        Budget,
                        Time,
                        Used,
                        NextAction,
                        DueDate,
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
                        ?,
                        ?
                    )
            ",
                task.category,
                task.finished,
                task.task_name,
                task.time_needed, // When creating a new task, save the initial time_needed estimate as time_budgeted
                task.time_needed,
                task.time_used,
                next_action_str,
                due_date_str,
                task.notes,
                date_added_str,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to insert Task into database")
                .last_insert_rowid();

            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            new_task = sqlx::query("
                SELECT *, rowid
                FROM worklist
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

    fn update_task(&self, task: Task){
        // These must be stored so that they are not dropped in-between
        // the calls to query! and .execute
        let next_action_str = DueDate::Date(task.next_action_date).to_string();
        let due_date_str = task.due_date.to_string();

        self.rt.block_on(async{
            sqlx::query!("
                UPDATE worklist
                SET
                    Category =    ?,
                    O =           ?,
                    Task =        ?,
                    Time =        ?,
                    Used =        ?,
                    NextAction =  ?,
                    DueDate =     ?,
                    Notes =       ?
                WHERE
                    rowid == ?
            ",
                task.category,
                task.finished,
                task.task_name,
                task.time_needed,
                task.time_used,
                next_action_str,
                due_date_str,
                task.notes,
                task.id,
            )
                .execute(&self.pool)
                .await
                .expect("Should be able to update task");
        })
    }

    fn delete_task(&self, task: Task){
        self.rt.block_on(async{
            sqlx::query!("
                DELETE
                FROM worklist
                WHERE rowid == ?
            ",
                task.id
            )
                .execute(&self.pool)
                .await
                .expect("Should be able do delete task");
        });
    }

    fn get_open_tasks(&self) -> Vec<Task>{
        let mut tasks: Vec<Task> = Vec::new();

        self.rt.block_on(async{
            // TODO this doesn't use query! because I'm too lazy to figure out how to annotate the
            // return type of query! to write an impl From<T> for Task
            tasks = sqlx::query("
                SELECT *, rowid
                FROM worklist
                WHERE O == 'O'
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get tasks")
                .into_iter()
                .map(|r: SqliteRow| Task::try_from(r)
                     .expect("Database should hold valid Tasks")
                ).collect()
        });

        tasks
    }

    fn get_categories(&self) -> Vec<String>{
        let mut categories: Vec<String> = Vec::new();

        self.rt.block_on(async{
            categories = sqlx::query!("
                SELECT DISTINCT Category
                FROM worklist
                ORDER BY Category
            ")
                .fetch_all(&self.pool)
                .await
                .expect("Should be able to get categories")
                .into_iter()
                .map(|r| r.Category.expect("Each category should be a string"))
                .collect()

        });

        categories
    }

    fn default_task(&self) -> Task{
        Task{
            category:         "Work".into(),
            finished:         "O".into(),
            task_name:        "".into(),
            _time_budgeted:   0,
            time_needed:      0,
            time_used:        0,
            next_action_date: today_date(),
            due_date:         DueDate::Date(today_date()),
            notes:            "".into(),
            id:               None,
            date_added:       today_date(),
        }
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
    m.add_class::<PyDueDate>().unwrap();
    m.add_class::<PyDueDateType>().unwrap();
    m.add_class::<DatabaseManager>().unwrap();
    Ok(())
}
