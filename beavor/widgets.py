#!/usr/bin/python3.11

import tkinter as tk
import datetime
from typing import Optional

from .backend import Project, Deliverable
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

class ProjectWindow(ScrollFrame):
    def __init__(self, parent):
        super().__init__(parent, "No project selected")
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
