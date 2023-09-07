#!/usr/bin/python3.11

import tkinter as tk
import datetime

from typing import Callable

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

class ContextMenuSpawner:
    """
    Spawns tk.Menus with a given configuration on right click on parent

    This CANNOT inherit from tk.Menu, even thought it feels like it should,
    because it needs to be able to spawn multiple menus

    This also cannot be inherited by a widget because it need to hold
    what the menu should look like
    """

    def __init__(self, parents: list[tk.Widget], menu_builder: Callable[[], tk.Menu]):

        self.parents = parents
        self.menu_builder = menu_builder

        for parent in parents:
            parent.bind("<3>", lambda evt: parent.after(1, lambda: self.make_context_menu(evt)))

    def make_context_menu(self, evt):
        self.ctx_menu = self.menu_builder()
        self.ctx_menu.post(evt.x_root, evt.y_root)

        self.funcid1 = self.parents[0].winfo_toplevel().bind("<1>", lambda _: self.destroy_context_menu(), "+")
        self.funcid2 = self.parents[0].winfo_toplevel().bind("<2>", lambda _: self.destroy_context_menu(), "+")
        self.funcid3 = self.parents[0].winfo_toplevel().bind("<3>", lambda _: self.destroy_context_menu(), "+")
        self.funcid4 = self.parents[0].winfo_toplevel().bind("<Configure>", lambda _: self.destroy_context_menu(), "+")

    def destroy_context_menu(self):

        self.parents[0].winfo_toplevel().unbind("<1>", self.funcid1)
        self.parents[0].winfo_toplevel().unbind("<2>", self.funcid2)
        self.parents[0].winfo_toplevel().unbind("<3>", self.funcid3)
        self.parents[0].winfo_toplevel().unbind("<Configure>", self.funcid4)
        self.ctx_menu.destroy()
