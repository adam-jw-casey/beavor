#!/usr/bin/python3.11

import tkinter as tk
import datetime
from typing import List, Optional

from .backend import green_red_scale, Task, Category, Project, Deliverable, PyDueDate, today_date, format_date, parse_date
from .ScrollFrame import ScrollFrame

class Timer(tk.Frame):

    def __init__(self, parent: tk.Frame | tk.LabelFrame, getSelectedTask, save, setUsedTime, notify):
        super().__init__(parent)

        self.notify = notify

        self.timeLabel = tk.Label(self, text=str(datetime.timedelta()))
        self.timeLabel.grid(row=0, column=1)

        self.timeButton = tk.Button(self, text="Start", command=lambda: self.toggleTimer(getSelectedTask()))
        self.timeButton.grid(row=0, column=0)
        self.timing = False

        self.save = save
        self.setUsedTime = setUsedTime

    def toggleTimer(self, selected_task) -> None:
        if not self.timing:
            self.start(selected_task)
        else:
            self.stop()
            self.save()

    def start(self, task) -> None:
        if task is None:
            self.notify("Cannot time an empty task")
            return

        self.timeButton.config(text="Stop")
        self.startTime = datetime.datetime.now()
        self.initialTime = datetime.timedelta(minutes=(task.time_used or 0))

        self.timing = True
        self._keep_displayed_time_updated()

    def stop(self):
        if self.timing:
            self.timeButton.config(text="Start")
            self.timing = False
            self.setUsedTime(round(self.timerVal.total_seconds()/60))

    def setTime(self, time: datetime.timedelta):
        self.timerVal = time
        self.timeLabel.config(text=str(time).split('.',2)[0])

    def _keep_displayed_time_updated(self):
        if self.timing:
            runTime = datetime.datetime.now() - self.startTime

            self.setTime((runTime + self.initialTime))

            self.after(1000, self._keep_displayed_time_updated)

    class EmptyTaskError(Exception):
      pass

class CategoryScroller(ScrollFrame):
    def __init__(self, parent, onRowClick):
        super().__init__(parent, "Projects")
        self.categoryRows = []
        self.onRowClick = onRowClick
        self.viewPort.grid_columnconfigure(0, weight=1)
    
    def showCategories(self, categories: List[Category]):
        for _ in range(len(self.categoryRows)):
            self.categoryRows.pop().destroy()
            
        for category in categories:
            self.add_category_row(category)

    def add_category_row(self, category):
        categoryRow = CategoryRow(self.viewPort, category, lambda c=category.name: self.onRowClick(c))
        categoryRow.pack(fill='x', side='bottom')
        self.categoryRows.append(categoryRow)

class CategoryRow(tk.Frame):
    def __init__(self, parent: tk.Frame, category: Category, select_project):
        def on_project_click(proj: Project):
            select_project(proj)
            self.unhighlight_all()

            next(filter(lambda pr: pr.project == proj, self.project_rows)).highlight()

        super().__init__(parent)

        self.category_name = category.name
        self.nameLabel = tk.Label(self, text=('▸ ' if len(category.projects) > 0 else '   ') + self.category_name)
        self.nameLabel.grid(sticky=tk.W)
        self.nameLabel.bind("<1>", lambda _: self.on_click())

        self.expanded = False

        self.project_rows: list[ProjectRow] = []
        for (i, proj) in enumerate(category.projects):
            pr = ProjectRow(self, proj, on_project_click)
            pr.grid(row=i+1, sticky=tk.W)
            pr.grid_forget()
            self.project_rows.append(pr)

    def expand(self):
        self.nameLabel.configure(text= '▾ ' + self.category_name)
        for pr in self.project_rows:
            pr.grid()

        self.expanded = True

    def collapse(self):
        self.nameLabel.configure(text= '▸ ' + self.category_name)
        for pr in self.project_rows:
            pr.unhighlight()
            pr.grid_forget()

        self.expanded = False

    def on_click(self):
        if len(self.project_rows) == 0:
            return

        if self.expanded:
            self.collapse()
        else:
            self.expand()

    def unhighlight_all(self):
        for pr in self.project_rows:
            pr.unhighlight()

class ProjectRow(tk.Frame):
    def __init__(self, parent: tk.Frame, project: Project, callback):
        super().__init__(parent)

        self.project = project

        self.nameLabel = tk.Label(self, text="-- " + project.name)
        self.nameLabel.pack(anchor=tk.W)
        self.nameLabel.bind("<1>", lambda _: callback(self.project))

        self.visible = [self, self.nameLabel]

    def highlight(self) -> None:
        for w in self.visible:
            w.config(bg="lightblue")

    def unhighlight(self) -> None:
        for w in self.visible:
            w.config(bg="white")

class ProjectWindow(tk.LabelFrame):
    def __init__(self, parent):
        super().__init__(parent)
        self.deliverable_rows: list[Deliverable] = []
        self.select_project(None)

    def select_project(self, proj: Optional[Project]):
        self.selected_project = proj
        self.config(text=proj.name if proj else "No project selected")

        for _ in range(len(self.deliverable_rows)):
            self.deliverable_rows.pop().destroy()

        if proj is None:
            return

        for deliverable in proj.deliverables:
            self.add_deliverable_row(deliverable)

    def add_deliverable_row(self, deliverable: Deliverable):
        deliverable_row = DeliverableRow(self, deliverable)
        deliverable_row.pack(fill='x', side='bottom')
        self.deliverable_rows.append(deliverable_row)

class DeliverableRow(tk.LabelFrame):
    def __init__(self, parent, deliverable: Deliverable):
        super().__init__(parent, text=deliverable.name)
        #self.notes_label=tk.Label(self, text=deliverable.notes)
        self.notes_label=tk.Label(self, text="lorem ipsum dolore")
        self.notes_label.pack(fill='x')
