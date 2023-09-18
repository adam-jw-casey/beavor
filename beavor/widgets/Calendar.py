import tkinter as tk
import tkinter.font
import datetime

from .SensibleReturnWidget import SensibleReturnWidget, LabelSR
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
        mark_vacation: Callable[[datetime.date], None] = lambda _: None,
        unmark_vacation: Callable[[datetime.date], None] = lambda _: None,
        on_click_date: Callable[[datetime.date], None] = lambda _: None,
        numweeks=4
    ):
        def context_menu_builder(date: datetime.date) -> tk.Menu:
            ctx = tk.Menu(self, tearoff=0)

            ctx.add_command(label="Mark vacation", command=lambda d=date: mark_vacation(d))
            ctx.add_command(label="Unmark vacation", command=lambda d=date: unmark_vacation(d))

            return ctx

        super().__init__(parentFrame, text="Calendar", padx=4, pady=4)

        self.numweeks = numweeks

        #Build the calendar out of labels
        self.calendar = []

        # TODO the days should have their own Widget class. Calendar shouldn't have so much low-level tinkering
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
                ).grid(row=2*week + 1, column=dayNum, padx=4, pady=4
                ).bind("<1>", lambda _, d=thisDay: on_click_date(d["Date"]))

                thisDay["LoadLabel"] = LabelSR(
                    self,
                ).grid(row=2*week + 2, column=dayNum, padx=4, pady=4
                ).bind("<1>", lambda _, d=thisDay: on_click_date(d["Date"]))

                ContextMenuSpawner([thisDay["DateLabel"], thisDay["LoadLabel"]], lambda d=thisDay: context_menu_builder(d["Date"]))

                thisWeek.append(thisDay)
            self.calendar.append(thisWeek)

    # todo this function isn't great but it seems to work
    def updateCalendar(self, schedule: Schedule) -> None:

        today = today_date()
        thisMonday = today - datetime.timedelta(days=today.weekday())
        hoursLeftToday = max(0, min(7, 16 - (datetime.datetime.now().hour + datetime.datetime.now().minute/60)))

        for week in range(self.numweeks):
            for day in range(5):
                thisDay = self.calendar[week][day]
                thisDate = thisMonday + datetime.timedelta(days=day, weeks=week)
                thisDay["Date"] = thisDate
                thisDay["DateLabel"].config(text=thisDate.strftime("%b %d"))

                # TODO should also highlight the day that is selected / filterred to
                if thisDate >= today:
                    if thisDate == today:
                        thisDay["DateLabel"].config(bg="lime")
                    elif not schedule.is_work_day(thisDay["Date"]):
                        thisDay["DateLabel"].config(bg="RoyalBlue")
                        thisDay["LoadLabel"].config(bg="gray85", text="")
                        continue
                    else:
                        thisDay["DateLabel"].config(bg="gray85")

                    hoursThisDay = schedule.workload_on_day(thisDate) / 60
                    thisDay["LoadLabel"]\
                      .config(
                          text=str(round(hoursThisDay,1)),
                          bg=green_red_scale(0,(8 if thisDate != today else max(0, hoursLeftToday)), hoursThisDay))
                else:
                    thisDay["DateLabel"].config(bg="gray85")
                    thisDay["LoadLabel"].config(text="", bg="gray85")
