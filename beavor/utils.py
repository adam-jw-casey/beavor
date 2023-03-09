import datetime
import numpy as np
import sqlite3
from enum import IntEnum
from typing import Optional

DATE_FORMAT = "%Y-%m-%d"

def greenRedScale(low: float, high: float, val: float) -> str:
  #linear interpolation bounded on [0,1]
  frac = max(0, min(1, (val - low) / (high - low)))
  if frac > 0.5:
    red = 255
    green = int((2-2*frac) * 255)
  else:
    red = int((2*frac) * 255)
    green = 255

  return "#{}{}00".format(str(hex(red)[2:]).rjust(2,'0'), str(hex(green)[2:]).rjust(2,'0'))

# takes strings "%Y-%m-%d"
# inclusive of start and end date
def workDaysBetween(d1: str | datetime.date, d2: str) -> int:
  return int(np.busday_count(d1, (YMDstr2date(d2) + datetime.timedelta(days=1))))

def YMDstr2date(dateString: str) -> datetime.date:
  return datetime.datetime.strptime(dateString, DATE_FORMAT).date()

def date2YMDstr(dateVar: datetime.date) -> str:
  return dateVar.strftime(DATE_FORMAT)

def todayStr() -> str:
  return date2YMDstr(todayDate())

def todayDate() -> datetime.date:
  return datetime.date.today()

class Task:
    def __init__(self, data: sqlite3.Row | dict):
        self.category:         str           = data["Category"]
        self.finished:         str           = data["O"]      # 'O' for open, 'X' for finished TODO would be better as a bool
        self.task_name:        str           = data["Task"]
        self._time_budgeted:    int           = data["Budget"] # in minutes
        self.time_needed:      int           = data["Time"]   # in minutes
        self.time_used:        int           = data["Used"]   # in minutes
        self.next_action_date: datetime.date = YMDstr2date(data["NextAction"])
        self.notes:            str           = data["Notes"]
        self.date_added:       datetime.date = YMDstr2date(data["DateAdded"])
        self.id:               Optional[int] = data["rowid"] if "rowid" in data.keys() else None
        self.due_date:         DueDate       = DueDate.fromString(data["DueDate"])

    @classmethod
    def default(cls):
        # TODO feels like it would be nicer to set default values in the database
        # creation query in DatabaseManager.createNewDatabase, but that would
        # require every the Task class to have access to the DataBase manager
        # class, which feels too cross-connected to me?
        return cls({
            "Category": "Work",
            "O": "O",
            "Task": "",
            "Budget": 0,
            "Time": 0,
            "Used": 0,
            "NextAction": todayStr(),
            "DueDate": todayStr(),
            "Notes": "",
            "id": None,
            "DateAdded": todayStr()
        })

class DueDateType(IntEnum):
    NONE = 0
    DATE = 2
    ASAP = 4

class DueDate:
    def __init__(self, date: DueDateType | datetime.date):
        match date:
            case DueDateType.DATE:
                raise ValueError("Must specify date")
            case DueDateType.NONE | DueDateType.ASAP:
                self.type = date
            case datetime.date():
                self.type = DueDateType.DATE
                self.date: datetime.date = date
            case _:
                raise TypeError(f"Invalid DueDateType: {type(date)}")

    @classmethod
    def fromString(cls, string):
        match string:
            case "NONE":
                return DueDate(DueDateType.NONE)
            case "ASAP":
                return DueDate(DueDateType.ASAP)
            case _:
                return DueDate(YMDstr2date(string))

    def __repr__(self) -> str:
        match self.type:
            case DueDateType.NONE:
                return "NONE"
            case DueDateType.DATE:
                return date2YMDstr(self.date)
            case DueDateType.ASAP:
                return "ASAP"
            case _:
                raise TypeError("This should never happen")

    def __eq__(self, other) -> bool:
        if self.type != other.type:
            return False
        else:
            return self.date == other.date

class DatabaseManager():
    def __init__(self, databasePath: str):
      self.conn = sqlite3.connect(databasePath)
      self.conn.row_factory = sqlite3.Row

      self.c = self.conn.cursor()
      self.cwrite = self.conn.cursor()

      self.getTasks([])
      self.headers = [description[0] for description in self.c.description]

    @classmethod
    def createNewDatabase(cls, path: str) -> None:
      conn = sqlite3.connect(path)
      cur  = conn.cursor()
      cur.execute("""
          CREATE TABLE worklist(
              Category  TEXT,
              O         TEXT,
              Task      TEXT,
              Budget    INTEGER,
              Time      INTEGER,
              Used      INTEGER,
              NextActionTEXT,
              DueDate   TEXT,
              Notes     TEXT,
              DateAdded TEXT)
      """)
      cur.close()

    def commit(self) -> None:
      self.conn.commit()

    # todo rewrite this
    #Loads the tasks by searching the database with the criteria specified
    def getTasks(self, criteria: list[str] = []) -> list[Task]:
      command = "SELECT *, rowid FROM worklist"

      if criteria:
        command += " WHERE "
        command += " AND ".join(criteria)

      command += " ORDER BY DueDate;"

      self.c.execute(command)

      return [Task(row) for row in self.c.fetchall()]

    def getTasks4Workload(self) -> list[Task]:
      self.cwrite.execute("SELECT * FROM worklist WHERE O == 'O' ORDER BY DueDate;")
      return self.cwrite.fetchall()

    #Updates the categories in the category filter
    def getCategories(self) -> list[str]:
      self.cwrite.execute("SELECT DISTINCT Category FROM worklist ORDER BY Category;")
      return [line["Category"] for line in self.cwrite.fetchall()]

    def deleteTask(self, task: Task) -> None:
      self.cwrite.execute("DELETE FROM worklist WHERE rowid == ?", [task.id])
      self.commit()

    def updateTask(self, task: Task) -> None:
        self.cwrite.execute(
            """
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
            """,
            (
                task.category,
                task.finished,
                task.task_name,
                task.time_needed,
                task.time_used,
                date2YMDstr(task.next_action_date),
                repr(task.due_date),
                task.notes,
                task.id
            )
        )

        self.commit()

    # Inserts a new Task into the database, then fetches the inserted task and returns it
    # Note that the returned Task is NOT identical to the passed one, because it will have 
    # a non-None rowid
    def createTask(self, task: Task) -> Task:
        self.cwrite.execute(
            """
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
            """,
            (
                task.category,
                task.finished,
                task.task_name,
                task.time_needed, # When creating a new task, save the initial time_needed estimate as time_budgeted
                task.time_needed,
                task.time_used,
                date2YMDstr(task.next_action_date),
                repr(task.due_date),
                task.notes,
                date2YMDstr(task.date_added)       
            )
        )
      
        self.cwrite.execute("SELECT * FROM worklist WHERE rowid == last_insert_rowid()")
        self.commit()
        return Task(self.cwrite.fetchall()[0])
