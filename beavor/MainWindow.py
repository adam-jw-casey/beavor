#!/usr/bin/python3.11

import sys

import tkinter as tk
import tkinter.messagebox
import tkinter.font

from typing import List, Optional
import json
import datetime

from .backend import DatabaseManager, Task, Schedule
from .widgets.SensibleReturnWidget import LabelSR
from .widgets.Calendar import Calendar
from .widgets.TaskScroller import TaskScroller
from .widgets.EditingPane import EditingPane

###########################################
#Readability / coding style / maintainability

# todo should add tests to make development smoother and catch bugs earlier

###########################################
#Nice-to-haves

# todo would be neat to have it build a daily schedule for me
# todo would be cool to support multi-step / project-type tasks
# todo integration to put tasks into Google/Outlook calendar would be cool or just have a way of marking a task as scheduled
# todo integration to get availability from Google/Outlook calendar to adjust daily workloads based on scheduled meetings
# todo Dark mode toggle (use .configure(bg='black') maybe? Or another better colour. Have to do it individually by pane though, self.root.configure() only does some of the background. Also probably have to change text colour too.)

###########################################

class Settings:
    def __init__(self, file_path: str):
        self.f = open(file_path, "r+")
        self.load()

    def load(self):
        self.data = json.load(self.f )
        return self.data

    def write(self):
        self.f.seek(0)
        json.dump(self.data, self.f)
        self.f.truncate()

    @classmethod
    def create_new(cls, settings_path: str):
        with open(settings_path, "w+") as f:
            json.dump({
                "font_size" : 10,
            }, f)

class MainWindow():
    def __init__(self, database_path: str, settings_path: str):
      self.os = sys.platform

      self.db = DatabaseManager(database_path)

      #Tkinter stuff
      self.root = tk.Tk()

      self.setupWindow(Settings(settings_path))

    def getSelectedTask(self):
        return self.selection

    ######################################################
    # GUI setup functions

    # Setup up the gui and load tasks
    def setupWindow(self, settings: Settings) -> None:
        if self.os == "linux":
            self.root.attributes('-zoomed', True)
        else:
            #win32
            self.root.state("zoomed")

        tk.font.nametofont("TkDefaultFont").configure(size=settings.data["font_size"])
        tk.font.nametofont("TkTextFont").configure(size=settings.data["font_size"])

        self.root.winfo_toplevel().title("WORKLIST Beta")

        # Frame to hold the tasklist display and associated frames and widgets

        self.db.get_open_tasks()

        self.task_list_scroller = TaskScroller(
            self.root,
            self.select_task
        ).grid(row=0, column=0, pady=4, padx=4, sticky=tk.N+tk.S+tk.E+tk.W)
        self.root.grid_columnconfigure(0, weight=1)

        # Editing interface
        self.editingPane = EditingPane(
            self.root,
            self.getSelectedTask,
            self.save_task,
            self.notify,
            self.db.get_categories,
            self.newTask,
            self.deleteTask,
            Task.default
        ).grid(row=0, column=1, padx=4, pady=4, sticky=tk.N+tk.S+tk.E+tk.W)
        self.root.grid_columnconfigure(1, weight=5)

        self.loadedTasks: List[Task] = []
        self.select_task(None)

        # Calendar
        self.calendar = Calendar(
            self.root,
            mark_vacation = self.add_vacation_day,
            unmark_vacation = self.remove_vacation_day,
            on_click_date = self.filter_to_date
        ).grid(row=0, column=2, pady=4, padx=4, sticky=tk.S+tk.E)

        self.messageLabel = LabelSR(
            self.root,
            text=""
        ).grid(column=1)

        self.root.grid_rowconfigure(0, weight=1)

        self.root.bind("<Control-q>", lambda _: self.root.destroy())
        self.root.bind("<Control-w>", lambda _: self.root.destroy())
        self.root.bind("<Control-n>", lambda _: self.newTask())

        self.refreshAll()

    ######################################################
    # GUI update functions

    # TODO implement a better state-management system so that all necessary widgets are updated when the database is updated
    def refreshTasks(self) -> None:
        #Remember which task was selected
        selected_rowid = self.selection.id if self.selection is not None else None

        self.loadedTasks = self.db.get_open_tasks()
        self.task_list_scroller.set_tasks(self.loadedTasks)

        match list(filter(lambda t: t.id == selected_rowid, self.loadedTasks)):
            case []:
                self.select_task(None)
            case [task]:
                self.select_task(task)
            case _:
                raise ValueError(f"This should never happen")

    def newTask(self, _=tk.Event) -> None:
      self.select_task(None)
      self.notify("New task created")

    def refreshAll(self) -> None:
      self.refreshTasks()

      schedule: Schedule = self.db.get_schedule()
      schedule.calculate_workloads(self.loadedTasks)

      self.calendar.updateCalendar(schedule)

    def notify(self, msg: str) -> None:
      self.messageLabel.config(text=msg)
      print(msg)

    ######################################################
    # Task functions

    #Deletes the task selected in the listbox from the database
    def deleteTask(self, task: Task) -> None:
      if(tk.messagebox.askyesno(
          title="Confirm deletion",
          message=f"Are you sure you want to delete '{task.name}'?")):
        self.db.delete_task(task)
        self.notify(f"Deleted '{task.name}'")

        self.newTask()
        self.refreshAll()

    #Save the current state of the entry boxes for that task
    def save_task(self, task: Task) -> None:
        self.editingPane.timer.stop()

        if self.selection is None:
            selected = self.db.create_task(task)
        else:
            self.db.update_task(task)
            selected = task

        # This prevents the "do you want to save first? prompt from appearing"
        self.editingPane.selection = None
        #Refresh the screen
        self.refreshAll()
        self.select_task(selected)

        self.notify("Task saved")

    def select_task(self,  task: Optional[Task]) -> None:
        if self.editingPane.tryShow(task):
            self.selection = task
            self.task_list_scroller.highlightTask(task)
        else:
            self.notify("Cancelled")

    def filter_to_date(self, date: datetime.date) -> None:
        self.task_list_scroller.show_by_availability_on_date(date)

    def add_vacation_day(self, date: datetime.date) -> None:
        self.db.add_vacation_day(date)
        self.refreshAll()

    def remove_vacation_day(self, date: datetime.date) -> None:
        self.db.delete_vacation_day(date)
        self.refreshAll()
