import tkinter as tk
import datetime

from .backend import parse_date, today_date, format_date
from .SensibleReturnWidget import EntrySR

class DateEntry(EntrySR):
  def __init__(self, parentFrame, notify):
    super().__init__(parentFrame)
    self.notify = notify
    self.bind("<Tab>", lambda _: self.convertDate)

  def convertDate(self) -> None:
    dateStr = self.get()
    convertedDate = ""

    try:
      parse_date(dateStr)
      convertedDate = dateStr
    except ValueError:
      try:
        #eg. Jan 1, 21
        convertedDate = format_date(datetime.datetime.strptime(dateStr, "%b %d, %y"))
      except ValueError:
        #Date string doesn't match
        try:
          #Try to add the current year
          #eg. Jan 1
          convertedDate = format_date(datetime.datetime.strptime(dateStr, "%b %d").replace(year = today_date().year))
        except ValueError:
          #Date really doesn't match
          self.notify("Can't match date format of {}".format(dateStr))
          return

    self.delete(0, tk.END)
    self.insert(0, convertedDate)

