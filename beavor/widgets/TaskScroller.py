import tkinter as tk
from typing import Optional
from pipe import filter
from datetime import date

from ..backend import Task, today_date, PyDueDate
from .ScrollFrame import ScrollFrame
from .SensibleReturnWidget import SensibleReturnWidget, CheckbuttonSR, LabelSR

# TODO - Add separators for tasks due today, this week, later

class TaskScroller(ScrollFrame, SensibleReturnWidget):
    def __init__(self, parent: tk.Frame | tk.LabelFrame | tk.Tk, onRowClick):
        super().__init__(parent, "Tasks")
        self.taskRows: list[TaskRow] = []
        self.tasks:    list[Task]    = []
        self.displayed_tasks:    list[Task] = []
        self.onRowClick = onRowClick

        self.show_available_only = tk.BooleanVar()
        self.selected_date: date = today_date()

        self.filter_indicator = LabelSR(
            self,
            text=today_date().strftime("%b %d")
        ).grid(row=1, column=0, sticky=tk.E+tk.W)

        self.available_only_button = CheckbuttonSR(
            self,
            text="Show only available tasks",
            variable=self.show_available_only,
            command = lambda: self.show_by_availability_on_date(None)
        ).grid(row=2, column=0, sticky=tk.E+tk.W)

    def show_by_availability_on_date(self, date: Optional[date]):
        if date is None:
            date = self.selected_date
        else:
            self.available_only_button.select()
            self.selected_date = date

        self.filter_indicator.config(text=date.strftime("%b %d"))

        if not self.show_available_only.get():
            self._show_tasks(self.tasks)
        else:
            if date == today_date():
                self._show_tasks(
                    self.tasks
                    | filter(
                        lambda t: t.next_action_date <= today_date()
                    )
                )
            else:
                self._show_tasks(
                    self.tasks
                    | filter(
                        lambda t: t.next_action_date <= date and t.due_date >= PyDueDate(date)
                    )
                )

    def set_tasks(self, tasks: list[Task]) -> None:
        self.tasks = tasks
        self.show_available_only.set(True)
        self._show_tasks(tasks)

    def _show_tasks(self, tasks: list[Task]) -> None:
        if self.displayed_tasks == tasks:
            return # TODO this might be preventing tasks from being hidden when their date is updated such that they are no longer availabe on the selected date

        self.displayed_tasks = tasks
        for _ in range(len(self.taskRows)):
            self.taskRows.pop().destroy()
            
        for (i, task) in enumerate(tasks):
            taskRow = TaskRow(
                self.viewPort,
                task,
                lambda t=task: self.onRowClick(t)
            ).grid(row=i, column=0, sticky= tk.W+tk.E, padx=4)
            self.taskRows.append(taskRow)

    def highlightTask(self, task: Optional[Task]) -> None:
        for tr in self.taskRows:
            if task is not None and tr.id == task.id:
                tr.highlight()
            else:
                tr.unhighlight()

# TODO bring in mouseover highlighting from new-schema sidebar.py ProjectRow
class TaskRow(tk.LabelFrame, SensibleReturnWidget):
    def __init__(self, parentFrame: tk.Frame, task, select):
        super().__init__(parentFrame)
        self.select = select
        self.id = task.id

        self.nameLabel = LabelSR(
            self,
            text=task.task_name,
            justify="left",
            anchor=tk.W
        ).pack(side=tk.TOP, fill=tk.X)
        self.bind('<Configure>', lambda _: self.nameLabel.config(wraplength=self.winfo_width()))

        self.categoryLabel = LabelSR(
            self,
            text=task.category,
            font=("helvetica", # todo why is font being set manually here
            8),
            anchor=tk.W
        ).pack(side=tk.BOTTOM, fill=tk.X)

        self.visible = [self, self.nameLabel, self.categoryLabel]
        for o in self.visible:
            o.bind("<1>", lambda _: self.select())

        self.unhighlight()

    def highlight(self) -> None:
        for w in self.visible:
            w.config(bg="lightblue")

    def unhighlight(self) -> None:
        for w in self.visible:
            w.config(bg="white")
