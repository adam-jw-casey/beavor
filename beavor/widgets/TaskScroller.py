import tkinter as tk
from typing import Optional

from ..backend import Task, today_date
from .ScrollFrame import ScrollFrame
from .SensibleReturnWidget import SensibleReturnWidget, CheckbuttonSR, LabelSR

class TaskScroller(ScrollFrame, SensibleReturnWidget):
    def __init__(self, parent: tk.Frame | tk.LabelFrame | tk.Tk, onRowClick):
        super().__init__(parent, "Tasks")
        self.taskRows: list[TaskRow] = []
        self.tasks: list[Task] = []
        self.onRowClick = onRowClick

        self.show_available_only = tk.BooleanVar()
        self.show_available_only.set(True)

        self.available_only_button = CheckbuttonSR(
            self,
            text="Show only available tasks",
            variable=self.show_available_only,
            command = lambda: self.showTasks(self.tasks)
        ).grid(row=1, column=0, sticky=tk.E+tk.W)

    def showTasks(self, tasks: list[Task]) -> None:
        self.tasks = tasks

        for _ in range(len(self.taskRows)):
            self.taskRows.pop().destroy()
            
        for (i, task) in enumerate(filter(lambda t: (not self.show_available_only.get()) or t.next_action_date <= today_date(), tasks)):
            taskRow = TaskRow(
                self.viewPort,
                task,
                lambda t=task: self.onRowClick(t)
            ).grid(row=i, column=0, sticky= tk.W+tk.E)
            self.taskRows.append(taskRow)

    def highlightTask(self, task: Optional[Task]) -> None:
        for tr in self.taskRows:
            if task is not None and tr.id == task.id:
                tr.highlight()
            else:
                tr.unhighlight()

class TaskRow(tk.LabelFrame, SensibleReturnWidget):
    def __init__(self, parentFrame: tk.Frame, task, select):
        super().__init__(parentFrame)
        self.select = select
        self.id = task.id

        self.nameLabel = LabelSR(
            self,
            text=task.task_name
        ).grid(row=0, column=0, sticky = tk.W + tk.E)

        self.categoryLabel = LabelSR(
            self,
            text=task.category,
            font=("helvetica",
            8)
        ).grid(row=1, column=0, sticky=tk.W)

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
