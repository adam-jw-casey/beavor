import tkinter as tk
import datetime

from .SensibleReturnWidget import SensibleReturnWidget, LabelSR
from .backend import green_red_scale, today_date, Task
from typing import Any

# todo put the next action / due date at a specific time?
# todo add buttons to scroll the calendar forward week-by-week
# todo Days of the week shown should be user-configurable (M-F vs. student schedule lol, or freelance).
# Set up the calendar display to show estimated workload each day for a several week forecast
class Calendar(tk.LabelFrame, SensibleReturnWidget):
    def __init__(self, parentFrame, parentFont):
        super().__init__(parentFrame, text="Calendar", padx=4, pady=4)

        self.numweeks = 4

        #Build the calendar out of labels
        self.calendar = []

        #Add day of week names at top, but won't change so don't save
        for i, day in enumerate(["Mon", "Tue", "Wed", "Thu", "Fri"]):
            LabelSR(
                self,
                font=parentFont + ("bold",),
                text=day
            ).grid(row=0, column=i, padx=4, pady=4)

        for week in range(self.numweeks):
            thisWeek = []
            for dayNum in range(5):
                thisDay: dict[str, Any] = {}
                # todo *Sometimes* this significantly slows boot time. Could maybe cut down on labels by having dates all in a row for each week, but lining up with loads could be tricky. First row changes colour, so could do each date row below the first as a multi-column label.
                #Alternate date labels and workloads
                thisDay["DateLabel"] = LabelSR(
                    self,
                    font=parentFont
                ).grid(row=2*week + 1, column=dayNum, padx=4, pady=4)
                thisDay["LoadLabel"] = LabelSR(
                    self,
                    font=parentFont
                ).grid(row=2*week + 2, column=dayNum, padx=4, pady=4)
                thisWeek.append(thisDay)
            self.calendar.append(thisWeek)

    # todo this function isn't great but it seems to work
    def updateCalendar(self, openTasks: list[Task]) -> None:
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
                    thisDay["DateLabel"].config(bg="#d9d9d9")
                if thisDate >= today:
                    hoursThisDay = self.getDayTotalLoad(thisDate, openTasks) / 60
                    thisDay["LoadLabel"]\
                      .config(
                          text=str(round(hoursThisDay,1)),
                          bg=green_red_scale(0,(8 if thisDate != today else max(0, hoursLeftToday)), hoursThisDay))
                else:
                    thisDay["LoadLabel"].config(text="", bg="#d9d9d9")

    def getDayTotalLoad(self, date: datetime.date, openTasks: list[Task]) -> float:
        return sum(
            task.workload_on_day(date)
            for task in openTasks
        )
