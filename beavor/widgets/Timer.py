import tkinter as tk
from .SensibleReturnWidget import FrameSR, LabelSR, ButtonSR
import datetime

class Timer(FrameSR):

    def __init__(self, parent: tk.Frame | tk.LabelFrame, getSelectedTask, save, setUsedTime, notify):
        super().__init__(parent)

        self.notify = notify

        self.timeLabel = LabelSR(
            self,
            text=str(datetime.timedelta())
        ).grid(row=0, column=1)

        self.timeButton = ButtonSR(
            self,
            text="Start",
            command=lambda: self.toggleTimer(getSelectedTask())
        ).grid(row=0, column=0)
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

