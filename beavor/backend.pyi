import datetime
from enum import Enum
from typing import Optional

def format_date(date: datetime.date) -> str:
    pass

def green_red_scale(low: float, high: float, val: float) -> str:
    pass

def parse_date(date: str) -> datetime.date:
    pass

def today_date() -> datetime.date:
    pass

def today_string() -> str:
    pass

class PyDueDate:
    def __init__(self, in_date: datetime.date):
        self.date_type: PyDueDateType
        date: Optional[datetime.date]

    @classmethod
    def parse(cls, s: str) -> PyDueDate:
        pass

class PyDueDateType(Enum):
    NONE = 0
    DATE = 1
    ASAP = 2

class Schedule:
    def is_available_on_day(self, task: Task, date: datetime.date) -> bool:
        pass

    def workload_on_day(self, date: datetime.date) -> int:
        pass

    def is_work_day(self, date: datetime.date) -> bool:
        pass

class Task:
    def __init__(self):
        self.category:         str
        self.finished:         bool
        self.name:             str
        self._time_budgeted:   int
        self.time_needed:      int
        self.time_used:        int
        self.notes:            str
        self.date_added:       datetime.date
        self.next_action_date: datetime.date
        self.due_date:         PyDueDate
        self.id:               int

    @staticmethod
    def default() -> Task:
        pass

class PyDatabaseManager:
    def __init__(self, database_path):
        pass

    @classmethod
    async def create_new_database(cls, database_path: str) -> None:
        pass

    async def create_task(self, task: Task) -> None:
        pass

    async def update_task(self, task: Task) -> None:
        pass

    async def delete_task(self, task: Task) -> None:
        pass

    async def get_open_tasks(self) -> list[Task]:
        pass

    async def get_categories(self) -> list[str]:
        pass

    async def try_update_holidays(self) -> None:
        pass

    async def add_vacation_day(self, date: datetime.date) -> None:
        pass

    async def delete_vacation_day(self, date: datetime.date) -> None:
        pass

    async def get_vacation_days(self) -> list[datetime.date]:
        pass

    async def get_holidays(self) -> list[datetime.date]:
        pass

    async def get_days_off(self) -> list[datetime.date]:
        pass

    async def get_schedule(self) -> Schedule:
        pass
