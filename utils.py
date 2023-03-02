import datetime
import numpy as np
import sqlite3

#Like .ljust, but truncates to length if necessary
def ljusttrunc(text: str, length: int) -> str:
  return text[:length].ljust(length)

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

#Surrounds the string inner with string outer, reversed the second time, and returns the result
def surround(inner: str, outer: str) -> str:
  return outer + inner + outer[::-1]

#Double up single quotes in a string
def escapeSingleQuotes(text) -> str:
  return "".join([c if c != "'" else c+c for c in text])

# Takes a string "YYYY-MM-DD"
def daysBetween(d1: str, d2: str) -> int:
  d1d = YMDstr2date(d1)
  d2d = YMDstr2date(d2)
  return (d2d - d1d).days

# takes strings "%Y-%m-%d"
# inclusive of start and end date
def workDaysBetween(d1: str | datetime.date, d2: str) -> int:
  return int(np.busday_count(d1, (YMDstr2date(d2) + datetime.timedelta(days=1))))

def YMDstr2date(dateString: str) -> datetime.date:
  return datetime.datetime.strptime(dateString, "%Y-%m-%d").date()

def date2YMDstr(dateVar: datetime.date) -> str:
  return dateVar.strftime("%Y-%m-%d")

def todayStr() -> str:
  return date2YMDstr(todayDate())

def todayDate() -> datetime.date:
  return datetime.date.today()

class Task:
    def __init__(self, data: sqlite3.Row | dict):
        self.category:   str    = data["Category"]
        self.o:          str    = data["O"]
        self.task:       str    = data["Task"]
        self.budget:     int    = data["Budget"]
        self.time:       int    = data["Time"]
        self.used:       int    = data["Used"]
        self.left:       int    = data["Left"]
        self.startDate:  str    = data["StartDate"]
        self.nextAction: str    = data["NextAction"]
        self.dueDate:    str    = data["DueDate"]
        self.flex:       str    = data["Flex"]
        self.daysLeft:   int    = data["DaysLeft"]
        self.totalLoad:  float  = data["TotalLoad"]
        self.load:       float  = data["Load"]
        self.notes:      str    = data["Notes"]
        self.dateAdded:  str    = data["DateAdded"]

class DatabaseManager():
  def __init__(self, databasePath: str):
    self.conn = sqlite3.connect(databasePath)
    self.conn.row_factory = sqlite3.Row

    self.c = self.conn.cursor()
    self.cwrite = self.conn.cursor()

    self.headers = [description[0] for description in self.c.description]

  @classmethod
  def createNewDatabase(cls, path: str) -> None:
    conn = sqlite3.connect(path)
    cur  = conn.cursor()
    # todo a better name for "Load" would be "CurrentLoad"
    cur.execute("""
        CREATE TABLE worklist(
            'Category'  TEXT,
            'O'         TEXT,
            'Task'      TEXT,
            'Budget'    INTEGER,
            'Time'      INTEGER,
            'Used'      INTEGER,
            'Left'      INTEGER,
            'StartDate' TEXT,
            'NextAction'TEXT,
            'DueDate'   TEXT,
            'Flex'      TEXT,
            'DaysLeft'  INTEGER,
            'TotalLoad' REAL,
            'Load'      REAL,
            'Notes'     TEXT,
            'DateAdded' TEXT)
    """)
    cur.close()

  def commit(self) -> None:
    self.conn.commit()

  #Loads the tasks by searching the database with the criteria specified
  def getTasks(self, criteria: list[str] =[]) -> list[dict]:
    #Super basic SQL injection check
    if True in [';' in s for s in criteria]:
      raise ValueError("; in SQL input!")

    command = "SELECT *, rowid FROM worklist"

    if criteria:
      command += " WHERE "
      command += " AND ".join(criteria)

    command += " ORDER BY DueDate;"

    self.c.execute(command)

    return self.c.fetchall()

  def getTasks4Workload(self) -> list[dict]:
    self.cwrite.execute("SELECT * FROM worklist WHERE O == 'O' ORDER BY DueDate;")
    return self.cwrite.fetchall()

  #Updates the categories in the category filter
  def getCategories(self) -> list[str]:
    self.cwrite.execute("SELECT DISTINCT Category FROM worklist ORDER BY Category;")
    return [line["Category"] for line in self.cwrite.fetchall()]

  def deleteByRowid(self, rowid: int) -> None:
    self.cwrite.execute("DELETE FROM worklist WHERE rowid == ?", [rowid])

  def deleteByNameCat(self, taskName: str, category: str) -> None:
    self.cwrite.execute("DELETE FROM worklist WHERE Task==? AND Category==? AND O='O'", [taskName, category])

  def checkSqlInput(self, sqlString) -> None:
    if type(sqlString) not in [int, float, type(None)]:
      #todo a better way of cleaning input
      badChars = [';']
      if any((c in badChars) for c in sqlString):
        raise ValueError("Bad SQL input: {}".format(sqlString))

  def updateTasks(self, criteria: list[str], changes: list[str]) -> None:
    for string in criteria + changes:
      self.checkSqlInput(string)

    command = "UPDATE worklist SET "
    command += ", ".join(changes)
    command += " WHERE "
    command += " AND ".join(criteria)
    command += ";"

    self.cwrite.execute(command)

  def createTask(self, headers: list[str], vals: list[str]) -> None:
    for string in headers + vals:
      self.checkSqlInput(string)

    cleanVals = []
    # Cleans quotes in SQL input
    for val in vals:
      try:
        cleanVals.append(surround(escapeSingleQuotes(str(val)), "'"))
      except TypeError:
        cleanVals.append(str(val))

    command = "INSERT INTO worklist ("

    command += ", ".join(headers)
    command +=  " ) VALUES ("

    command += ", ".join(cleanVals)
    command += " );"

    self.cwrite.execute(command)

