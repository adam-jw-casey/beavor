#!/usr/bin/python3.11

import tkinter as tk
import datetime

from typing import Callable, Optional

class Timer(tk.Frame):
    def __init__(self, parent: tk.Frame | tk.LabelFrame, getSelectedTask, save, setUsedTime, notify):
        super().__init__(parent)

        self.notify = notify

        self.timeLabel = tk.Label(self, text=str(datetime.timedelta()))
        self.timeLabel.grid(column=0)

        self.timeButton = tk.Button(self, text="Start", command=lambda: self.toggleTimer(getSelectedTask()))
        self.timeButton.grid(column=1)
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
        self.ctx_menu.bind("<1>", lambda _: self.click_tracker.unbind(), "+")
        self.ctx_menu.bind("<2>", lambda _: self.click_tracker.unbind(), "+")
        self.ctx_menu.bind("<3>", lambda _: self.click_tracker.unbind(), "+")

        self.click_tracker = TrackOutsideClick(self.parents)
        self.click_tracker.bind(self.destroy_context_menu)

    def destroy_context_menu(self):
        self.ctx_menu.destroy()
        self.click_tracker.unbind()

class EditableLabel(tk.Frame):
    def __init__(self, parent, text: str, edit_text: Callable[[str], None]):
        super().__init__(parent)
        self.text = text
        self.edit_text = edit_text

        self.label = tk.Label(self, text=text)
        self.label.bind("<Double-Button-1>", lambda _: self.edit())
        self.label.grid(sticky=tk.N+tk.S+tk.E+tk.W)

        self.edit_box = tk.Entry(self)
        self.edit_box.bind("<Return>", lambda _: self.save())
        self.click_tracker = TrackOutsideClick([self.edit_box])

    def bind(self, *args, **kwargs):
        super().bind(*args, **kwargs)
        self.label.bind(*args, **kwargs)
        self.edit_box.bind(*args, **kwargs)

    def save(self):
        self.text = self.edit_box.get()
        self.edit_text(self.text)
        self.edit_box.grid_forget()

        self.label.config(text=self.text)
        self.label.grid(sticky=tk.N+tk.S+tk.E+tk.W)

        self.click_tracker.unbind()

    def edit(self):
        def select_all():
            # select text
            self.edit_box.select_range(0, 'end')
            # move cursor to the end
            self.edit_box.icursor('end')


        self.label.grid_forget()
        self.edit_box.grid(sticky=tk.N+tk.S+tk.E+tk.W)
        self.edit_box.delete(0, tk.END)
        self.edit_box.insert(0, self.text)

        self.edit_box.focus_force()
        select_all()

        self.click_tracker.bind(self.save, ignore_self_click=True)

class TrackOutsideClick:
    """
    Register when click events occur outside of a given widget
    """
    def __init__(self, domain: list[tk.Widget]):
        self.funcid1: str
        self.funcid2: str
        self.funcid3: str
        self.funcid4: str

        self.domain: list[tk.Widget] = domain
        self.bound: bool = False

    def bind(self, callback: Callable[[], Optional[str]], ignore_self_click=False):
        # Only allow a single binding, to avoid losing ids of old bindings
        # and leaving them dangling
        if self.bound:
            self.unbind()
        self.bound = True

        # Optionally ignore clicks that occur on domain widget
        wrapped_callback: Callable[[tk.Event], Optional[str]]
        if ignore_self_click:
            wrapped_callback = lambda e: (callback() if e.widget not in self.domain else None)
        else:
            wrapped_callback = lambda _: callback()

        self.funcid1 = self.domain[0].winfo_toplevel().bind("<1>", wrapped_callback, "+")
        self.funcid2 = self.domain[0].winfo_toplevel().bind("<2>", wrapped_callback, "+")
        self.funcid3 = self.domain[0].winfo_toplevel().bind("<3>", wrapped_callback, "+")
        self.funcid4 = self.domain[0].winfo_toplevel().bind("<Configure>", lambda e: (callback() if e.widget == self.domain[0].winfo_toplevel() else None), "+")

    def unbind(self):
        self.domain[0].winfo_toplevel().unbind("<1>", self.funcid1)
        self.domain[0].winfo_toplevel().unbind("<2>", self.funcid2)
        self.domain[0].winfo_toplevel().unbind("<3>", self.funcid3)
        self.domain[0].winfo_toplevel().unbind("<Configure>", self.funcid4)

        self.bound = False
