import tkinter as tk
import tkinter.font
import datetime

from .SensibleReturnWidget import SensibleReturnWidget, LabelSR
from ..backend import green_red_scale, today_date, Task, Schedule
from typing import Any, Callable

# todo put the next action / due date at a specific time?
# todo add buttons to scroll the calendar forward week-by-week
# todo Days of the week shown should be user-configurable (M-F vs. student schedule lol, or freelance).

# Set up the calendar display to show estimated workload each day for a several week forecast
class Calendar(tk.LabelFrame, SensibleReturnWidget):
    def __init__(self, parentFrame, schedule: Schedule, on_click_date: Callable[[datetime.date], None]=lambda _: None, numweeks=4):
        super().__init__(parentFrame, text="Calendar", padx=4, pady=4)

        self.schedule: Schedule = schedule
        self.numweeks = numweeks

        #Build the calendar out of labels
        self.calendar = []

        #Add day of week names at top, but won't change so don't save
        for i, day in enumerate(["Mon", "Tue", "Wed", "Thu", "Fri"]):
            LabelSR(
                self,
                font=tk.font.nametofont("TkDefaultFont"),
                text=day
            ).grid(row=0, column=i, padx=4, pady=4)

        for week in range(self.numweeks):
            thisWeek = []
            for dayNum in range(5):
                thisDay: dict[str, Any] = {}
                #Alternate date labels and workloads
                thisDay["DateLabel"] = LabelSR(
                    self,
                ).grid(row=2*week + 1, column=dayNum, padx=4, pady=4)
                thisDay["DateLabel"].bind("<1>", lambda _, d=thisDay: on_click_date(d["Date"]))

                thisDay["LoadLabel"] = LabelSR(
                    self,
                ).grid(row=2*week + 2, column=dayNum, padx=4, pady=4)
                thisDay["LoadLabel"].bind("<1>", lambda _, d=thisDay: on_click_date(d["Date"]))

                thisWeek.append(thisDay)
            self.calendar.append(thisWeek)

    # todo this function isn't great but it seems to work
    def updateCalendar(self, openTasks: list[Task]) -> None:
        self.schedule.calculate_workloads(openTasks)

        today = today_date()
        thisMonday = today - datetime.timedelta(days=today.weekday())
        hoursLeftToday = max(0, min(7, 16 - (datetime.datetime.now().hour + datetime.datetime.now().minute/60)))
        for week in range(self.numweeks):
            for day in range(5):
                thisDay = self.calendar[week][day]
                thisDate = thisMonday + datetime.timedelta(days=day, weeks=week)
                thisDay["Date"] = thisDate
                thisDay["DateLabel"].config(text=thisDate.strftime("%b %d"))

                if thisDate == today:
                    thisDay["DateLabel"].config(bg="lime")
                else:
                    thisDay["DateLabel"].config(bg="gray85")

                if thisDate >= today:
                    hoursThisDay = self.schedule.workload_on_day(thisDate) / 60
                    thisDay["LoadLabel"]\
                      .config(
                          text=str(round(hoursThisDay,1)),
                          bg=green_red_scale(0,(8 if thisDate != today else max(0, hoursLeftToday)), hoursThisDay))
                else:
                    thisDay["LoadLabel"].config(text="", bg="gray85")
