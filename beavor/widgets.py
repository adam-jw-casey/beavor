#!/usr/bin/python3.11

import tkinter as tk
import tkinter.messagebox
import tkinter.ttk as ttk
import datetime
from typing import List, Any, Optional

from beavor.backend import green_red_scale, Task, Category, PyDueDate, today_date, format_date, parse_date
from beavor.ScrollFrame import ScrollFrame

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
    def __init__(self, parent: tk.Frame | tk.LabelFrame, onRowClick):
        super().__init__(parent)
        self.categoryRows = []
        self.onRowClick = onRowClick
        self.viewPort.grid_columnconfigure(0, weight=1)
    
    def showCategories(self, categories: List[Category]):
        for _ in range(len(self.categoryRows)):
            self.categoryRows.pop().destroy()
            
        for category in categories:
            self.add_category(category)

    def add_category(self, category):
        categoryRow = CategoryRow(self.viewPort, category, lambda c=category.name: self.onRowClick(c))
        categoryRow.pack(fill='x', side='bottom')
        self.categoryRows.append(categoryRow)

   
class CategoryRow(tk.Frame):
    def __init__(self, parent: tk.Frame, category: Category, callback):
        super().__init__(parent)
        self.category_name = category.name
        self.nameLabel = tk.Label(self, text='▶ '+ self.category_name)
        self.nameLabel.grid(row=0, column=0, sticky = tk.W)

        self.visible = [self, self.nameLabel]

        self.callback = callback
        for w in self.visible:
            w.bind("<1>", lambda _: self.on_click())

    def expand(self):
        self.nameLabel.configure(text= '▼ ' + self.category_name)

    def collapse(self):
        self.nameLabel.configure(text= '▶ ' + self.category_name)

    def on_click(self):
        self.highlight()
        self.expand()
        self.callback()

    def highlight(self) -> None:
        for w in self.visible:
            w.config(bg="lightblue")

    def unhighlight(self) -> None:
        for w in self.visible:
            w.config(bg="white")
