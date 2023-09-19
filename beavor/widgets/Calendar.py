import tkinter as tk
import tkinter.font
import datetime

from .SensibleReturnWidget import SensibleReturnWidget, LabelSR, FrameSR
from ..utils.ContextMenuSpawner import ContextMenuSpawner
from ..backend import green_red_scale, today_date, Schedule
from typing import Any, Callable

# todo put the next action / due date at a specific time?
# todo add buttons to scroll the calendar forward week-by-week
# todo Days of the week shown should be user-configurable (M-F vs. student schedule lol, or freelance).

# Set up the calendar display to show estimated workload each day for a several week forecast
class Calendar(tk.LabelFrame, SensibleReturnWidget):
    def __init__(
        self,
        parentFrame,
        mark_vacation:   Callable[[datetime.date], None],
        unmark_vacation: Callable[[datetime.date], None],
        on_click_date:   Callable[[datetime.date], None],
        numweeks=4
    ):
        super().__init__(parentFrame, text="Calendar", padx=4, pady=4)

        self.numweeks = numweeks

        #Build the calendar out of labels
        self.calendar: list[list[DayDisplay]] = []

        # Add day of week headers
        for i, day in enumerate(["Mon", "Tue", "Wed", "Thu", "Fri"]):
            LabelSR(
                self,
                font=tk.font.nametofont("TkDefaultFont"),
                text=day
            ).grid(row=0, column=i, padx=4, pady=4)

        for week in range(self.numweeks):
            thisWeek:list[DayDisplay] = []

            for dayNum in range(5):
                day = DayDisplay(
                    self,
                    mark_vacation,
                    unmark_vacation,
                    on_click_date
                ).grid(row=week + 1, column=dayNum, padx=4, pady=4)
                thisWeek.append(day)

            self.calendar.append(thisWeek)

    def updateCalendar(self, schedule: Schedule) -> None:
        today = today_date()
        thisMonday = today - datetime.timedelta(days=today.weekday())
        hoursLeftToday = max(0, min(7, 16 - (datetime.datetime.now().hour + datetime.datetime.now().minute/60)))

        for week in range(self.numweeks):
            for week_day in range(5):
                day = self.calendar[week][week_day]
                date = thisMonday + datetime.timedelta(days=week_day, weeks=week)

                day.update(schedule, date, today, hoursLeftToday)

class DayDisplay(FrameSR):
    def __init__(
        self,
        parent,
        mark_vacation:   Callable[[datetime.date], None],
        unmark_vacation: Callable[[datetime.date], None],
        on_click_date:   Callable[[datetime.date], None]
    ):
        def context_menu_builder(date: datetime.date) -> tk.Menu:
            ctx = tk.Menu(self, tearoff=0)

            ctx.add_command(label="Mark vacation",   command=lambda d=date: mark_vacation(d))
            ctx.add_command(label="Unmark vacation", command=lambda d=date: unmark_vacation(d))

            return ctx

        super().__init__(parent)

        self.date: datetime.date

        #Alternate date labels and workloads
        self.date_label: LabelSR = LabelSR(
            self
        ).pack(side="top"
        ).bind("<1>", lambda _, d=self: on_click_date(d.date))

        self.load_label: LabelSR = LabelSR(
            self
        ).pack(side="bottom"
        ).bind("<1>", lambda _, d=self: on_click_date(d.date))

        ContextMenuSpawner([self.date_label, self.load_label], lambda d=self: context_menu_builder(d.date))

    def update(self, schedule: Schedule, date: datetime.date, today: datetime.date, hoursLeftToday: float):
        self.date = date
        self.date_label.config(text=date.strftime("%b %d"))

        # TODO should also highlight the day that is selected / filtered to
        if date >= today:
            if date == today:
                self.date_label.config(bg="lime")
            elif not schedule.is_work_day(date):
                self.date_label.config(bg="RoyalBlue")
                self.load_label.config(bg="gray85", text="")
                return
            else:
                self.date_label.config(bg="gray85")

            hoursThisDay = schedule.workload_on_day(date) / 60
            self.load_label\
              .config(
                  text=str(round(hoursThisDay,1)),
                  bg=green_red_scale(0,(8 if date != today else max(0, hoursLeftToday)), hoursThisDay))
        else:
            self.date_label.config(bg="gray85")
            self.load_label.config(text="", bg="gray85")
