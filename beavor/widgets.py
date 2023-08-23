#!/usr/bin/python3.11

import tkinter as tk
import tkinter.messagebox
import tkinter.ttk as ttk
import datetime
import sys
from typing import List, Any, Optional

from .backend import green_red_scale, DatabaseManager, Task, PyDueDate, today_date, format_date, parse_date
from .ScrollFrame import ScrollFrame

###########################################
#Readability / coding style / maintainability

# todo should add tests to make development smoother and catch bugs earlier
# todo go through and make sure exceptions are being handled in a reasonable place and manner

###########################################
#Nice-to-haves

# todo would be neat to have it build a daily schedule for me
# todo would be cool to support multi-step / project-type tasks
# todo integration to put tasks into Google/Outlook calendar would be cool or just have a way of marking a task as scheduled
# todo integration to get availability from Google/Outlook calendar to adjust daily workloads based on scheduled meetings
# todo Dark mode toggle (use .configure(bg='black') maybe? Or another better colour. Have to do it individually by pane though, self.root.configure() only does some of the background. Also probably have to change text colour too.)
# todo User-adjustable font/font size
    # todo user-customizable settings (like font size, calendar colourscale) -> This could write to external file, read at startup?

###########################################

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

class DateEntry(tk.Entry):
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

class CompletingComboBox(ttk.Combobox):
    def __init__(self, parent, getOptions):
        super().__init__(parent)

        self.bind("<FocusOut>", lambda _: self.selection_clear())
        self.bind("<KeyRelease>", lambda event: self._completeBox(event, getOptions))
        self.bind("<Return>", lambda _: self.icursor(tk.END))

        self.config(values=getOptions())

    def _completeBox(self, event: tk.Event, getSourceList) -> None:

      #Don't run when deleting, or when shift is released
      if event.keysym in ["BackSpace", "Shift_L", "Shift_R"]:
          return

      cursorPos: int = self.index(tk.INSERT)
      current: str = self.get()[:]

      #Don't run if self is empty, or cursor is not at the end
      if current and cursorPos == len(self.get()):
        # Find all options beginning with the current string
        options: List[str] = list(filter(lambda s: s.find(current) == 0, getSourceList()))

        if options:
            # Find longest shared leading (from cursor) substring among matching options
            i: int = len(current)-1
            while i < min([len(o) for o in options]):
                if len(set([option[i] for option in options])) != 1:
                    break
                i += 1

            # If found a match
            if i > len(current):
              self.insert(tk.END, options[0][cursorPos:i+1])

            self.select_range(cursorPos, tk.END)
            self.icursor(tk.END)

class EditingPane(tk.LabelFrame):
    def __init__(self, parent, getSelectedTask, save, notify, get_categories, newTask, deleteTask, getDefaultTask):
        def canBeInt(d, i, P, s, S, v, V, W) ->  bool:
            try:
                int(S)
                return True
            except ValueError:
                return False

        super().__init__(parent, text="Edit")

        self.save = lambda: save(self._createTaskFromInputs())
        self.get_categories = get_categories
        self.notify = notify

        self.selection: Optional[Task] = None

        self.getDefaultTask = getDefaultTask

        int_validation = (self.register(canBeInt),
                '%d', '%i', '%P', '%s', '%S', '%v', '%V', '%W')

        # Entry boxes and labels
        eframe = tk.Frame(self)
        eframe.grid(row=0, column=0)

        # For save button, etc. below entry boxes
        self.entryButtonFrame = tk.Frame(self)
        self.entryButtonFrame.grid(row=1, column=0)

        # Timer and its button
        self.timer = Timer(self.entryButtonFrame, getSelectedTask, self.save, lambda time: self._overwriteEntryBox(self.usedBox, time), notify)
        self.timer.grid(row=0, column=1)

        #Setup the lower half of the window
        self.categoryLabel = tk.Label(eframe, text= "Category")
        self.categoryBox = CompletingComboBox(eframe, get_categories)

        self.taskNameLabel = tk.Label(eframe, text="Task Name")
        self.taskNameBox = tk.Entry(eframe)

        self.timeLabel = tk.Label(eframe, text="Time Needed")
        self.timeBox = tk.Entry(eframe, validate="key", validatecommand=int_validation)

        self.usedLabel = tk.Label(eframe, text="Time Used")
        self.usedBox = tk.Entry(eframe, validate="key", validatecommand=int_validation)

        self.nextActionLabel = tk.Label(eframe, text="Next Action")
        self.nextActionBox = DateEntry(eframe, notify)

        self.dueDateLabel = tk.Label(eframe, text="Due Date")
        self.dueDateBox = DateEntry(eframe, notify)

        self.notesLabel = tk.Label(eframe, text="Notes")
        self.notesBox = tk.Text(eframe, wrap="word")

        for i, [label, widget] in enumerate([
            [self.categoryLabel, self.categoryBox],
            [self.taskNameLabel, self.taskNameBox],
            [self.timeLabel, self.timeBox],
            [self.usedLabel, self.usedBox],
            [self.nextActionLabel, self.nextActionBox],
            [self.dueDateLabel, self.dueDateBox],
            [self.notesLabel, self.notesBox]
                ]):
            label.grid(sticky="W", row=i, column=0)
            widget.grid(sticky="NW",row=i, column=1, pady=1)
            widget.config(width=50)


        self.doneIsChecked = tk.StringVar()
        self.doneCheckBox = tk.Checkbutton(self.entryButtonFrame,
                                           text="Done",
                                           variable=self.doneIsChecked,
                                           onvalue="X",
                                           offvalue="O")
        self.doneCheckBox.grid(row=0, column=0)
        self.doneCheckBox.deselect()

        #Add buttons to interact
        self.saveButton = tk.Button(self.entryButtonFrame, text="Save", command=self.save)
        self.saveButton.grid(row=0, column=2)

        self.newTaskButton = tk.Button(self.entryButtonFrame, text="New", command=newTask)
        self.newTaskButton.grid(row=0, column=3)

        self.deleteButton = tk.Button(self.entryButtonFrame,
                                      text="Delete",
                                      command = lambda: deleteTask(self.selection))
        self.deleteButton.grid(row=0, column=4)

    def tryShow(self, task: Optional[Task]) -> bool:
      self.categoryBox.config(values=self.get_categories())

      if self.selection is not None:
          self.timer.stop()

          if self._nonTrivialChanges():
              match self._askSaveChanges(self.selection.task_name):
                  case True:
                      self.save()
                  case False:
                      pass
                  case None:
                      return False

      self.deleteButton.config(state="normal" if task is not None else "disabled")

      self.selection = task
      task = task or self.getDefaultTask()
      assert(task is not None) # Just to make the linter happy, this is unnecessary because of the line above

      self._overwriteEntryBox(self.categoryBox,     task.category)
      self._overwriteEntryBox(self.taskNameBox,     task.task_name)
      self._overwriteEntryBox(self.timeBox,         task.time_needed)
      self._overwriteEntryBox(self.usedBox,         task.time_used)
      self._overwriteEntryBox(self.dueDateBox,      task.due_date)
      self._overwriteEntryBox(self.nextActionBox,   task.next_action_date)
      self._overwriteEntryBox(self.notesBox,        task.notes)
      self.timer.setTime(datetime.timedelta(minutes=(task.time_used or 0)))
      self.doneIsChecked.set(task.finished)

      return True

    # todo this needs better input validation
    def _createTaskFromInputs(self) -> Task:
        self.timer.stop()

        task: Task             = self.selection or self.getDefaultTask()

        try:
            task.category          = self.categoryBox.get()
            task.task_name         = self.taskNameBox.get()
            task.time_needed       = int(self.timeBox.get())
            task.time_used         = int(self.usedBox.get())
            task.next_action_date  = parse_date(self.nextActionBox.get())
            task.notes             = self.notesBox.get('1.0', 'end')[:-1]
            task.due_date          = PyDueDate.parse(self.dueDateBox.get())
            task.finished          = self.doneIsChecked.get()
        except ValueError as e:
            # On any input validation errors, notify the user and print error - todo not pretty but better than nothing
            self.notify(e.__str__())
            raise(e)

        return task

    def _nonTrivialChanges(self) -> bool:
        if self.selection is None:
            return False
        else:
            t1 = self.selection
            t2 = self._createTaskFromInputs()

            return (
                t1.category         != t2.category         or
                t1.task_name        != t2.task_name        or
                t1.time_needed      != t2.time_needed      or
                t1.time_used        != t2.time_used        or
                t1.next_action_date != t2.next_action_date or
                t1.notes            != t2.notes            or
                t1.due_date         != t2.due_date
            )

    def _clearEntryBoxes(self) -> None:
        self.doneIsChecked.set("O")
        self.timer.setTime("0:00:00")
        for w in [self.categoryBox, self.taskNameBox, self.timeBox, self.usedBox, self.dueDateBox, self.nextActionBox, self.notesBox]:
          self._overwriteEntryBox(w, "")

    def _overwriteEntryBox(self, entry: ttk.Combobox | tk.Text | tk.Entry | DateEntry, text) -> None:
      #Check if we need to temporarily enable the box
      changeFlag = (entry["state"] == "readonly")
      if changeFlag:
        entry.config(state="normal")

      # todo a bit janky
      try:
        entry.delete('1.0','end')# tk.text
      except tk.TclError:
        entry.delete(0,'end')# tk.Entry
      entry.insert('end', text)

      #Switch back to the original state
      if changeFlag:
        entry.config(state=tk.DISABLED)

    def _askSaveChanges(self, taskName: str) -> bool:
        return tk.messagebox.askyesnocancel(
            title="Save before switching?",
            message=f"Do you want to save your changes to '{taskName}' before switching?"
        )

# todo put the next action / due date at a specific time?
# todo add buttons to scroll the calendar forward week-by-week
# todo Days of the week shown should be user-configurable (M-F vs. student schedule lol, or freelance).
# Set up the calendar display to show estimated workload each day for a several week forecast
class Calendar(tk.LabelFrame):
  def __init__(self, parentFrame, parentFont):
    super().__init__(parentFrame, text="Calendar", padx=4, pady=4)

    self.numweeks = 4

    #Build the calendar out of labels
    self.calendar = []

    #Add day of week names at top, but won't change so don't save
    for i, day in enumerate(["Mon", "Tue", "Wed", "Thu", "Fri"]):
      tk.Label(self, font=parentFont + ("bold",), text=day).grid(row=0, column=i, padx=4, pady=4)

    for week in range(self.numweeks):
      thisWeek = []
      for dayNum in range(5):
        thisDay: dict[str, Any] = {}
        # todo *Sometimes* this significantly slows boot time. Could maybe cut down on labels by having dates all in a row for each week, but lining up with loads could be tricky. First row changes colour, so could do each date row below the first as a multi-column label.
        #Alternate date labels and workloads
        thisDay["DateLabel"] = tk.Label(self, font=parentFont)
        thisDay["DateLabel"].grid(row=2*week + 1, column=dayNum, padx=4, pady=4)
        thisDay["LoadLabel"] = tk.Label(self, font=parentFont)
        thisDay["LoadLabel"].grid(row=2*week + 2, column=dayNum, padx=4, pady=4)
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

class TaskScroller(ScrollFrame):
    def __init__(self, parent: tk.Frame | tk.LabelFrame, onRowClick):
        super().__init__(parent, "Tasks")
        self.taskRows = []
        self.onRowClick = onRowClick

    def showTasks(self, tasks: list[Task]) -> None:
        for _ in range(len(self.taskRows)):
            self.taskRows.pop().destroy()
            
        for (i, task) in enumerate(tasks):
            taskRow = TaskRow(self.viewPort, task, lambda t=task: self.onRowClick(t))
            taskRow.grid(row=i, column=0, sticky= tk.W+tk.E)
            self.taskRows.append(taskRow)

    def highlightTask(self, task: Optional[Task]) -> None:
        for tr in self.taskRows:
            if task is not None and tr.id == task.id:
                tr.highlight()
            else:
                tr.unhighlight()

class TaskRow(tk.LabelFrame):
    def __init__(self, parentFrame: tk.Frame, task, select):
        super().__init__(parentFrame)
        self.select = select
        self.id = task.id

        self.nameLabel = tk.Label(self, text=task.task_name)
        self.nameLabel.grid(row=0, column=0, sticky = tk.W + tk.E)

        self.categoryLabel = tk.Label(self, text=task.category, font=("helvetica", 8))
        self.categoryLabel.grid(row=1, column=0, sticky=tk.W)

        self.visible = [self, self.nameLabel, self.categoryLabel]
        for o in self.visible:
            o.bind("<1>", lambda _: self.select())

        self.unhighlight()

    def highlight(self) -> None:
        for w in self.visible:
            w.config(bg="lightblue")

    def unhighlight(self) -> None:
        for w in self.visible:
            w.config(bg="white")

class WorklistWindow():

    def __init__(self, databasePath: str):
      self.os = sys.platform

      self.db = DatabaseManager(databasePath)

      #Tkinter stuff
      self.root = tk.Tk()

      self.setupWindow()

    def getSelectedTask(self):
        return self.selection

    ######################################################
    # GUI setup functions

    # Setup up the gui and load tasks
    def setupWindow(self) -> None:

      if self.os == "linux":
        self.root.attributes('-zoomed', True)
        self.font = ("Liberation Mono", 10)
      else:
        #win32
        self.root.state("zoomed")
        self.font = ("Courier", 10)

      self.root.winfo_toplevel().title("WORKLIST Beta")

      # Frame to hold the tasklist display and associated frames and widgets

      self.db.get_open_tasks()

      # Editing interface
      self.editingPane = EditingPane(self.root, self.getSelectedTask, self.save, self.notify, self.db.get_categories, self.newTask, self.deleteTask, self.db.default_task)
      self.editingPane.grid(row=0, column=1, padx=4, pady=4)

      self.scroller = TaskScroller(self.root, self.select)
      self.scroller.grid(row=0, column=0, pady=4, padx=4, sticky=tk.N+tk.S+tk.E+tk.W)

      self.loadedTasks: List[Task] = []
      self.select(None)

      # Calendar
      self.calendar = Calendar(self.root, self.font)
      self.calendar.grid(row=0, column=2, pady=4, padx=4, sticky=tk.S)

      self.messageLabel = tk.Label(self.root, text="")
      self.messageLabel.grid(column=1)

      self.root.bind("<Control-q>", lambda _: self.root.destroy())
      self.root.bind("<Control-w>", lambda _: self.root.destroy())
      self.root.bind("<Control-n>", lambda _: self.newTask())

      self.refreshAll()

    ######################################################
    # GUI update functions

    def refreshTasks(self) -> None:
        #Remember which task was selected
        selected_rowid = self.selection.id if self.selection is not None else None

        self.loadedTasks = self.db.get_open_tasks()
        self.scroller.showTasks(self.loadedTasks)

        match list(filter(lambda t: t.id == selected_rowid, self.loadedTasks)):
            case []:
                self.select(None)
            case [task]:
                self.select(task)
            case _:
                raise ValueError(f"This should never happen")

    def newTask(self, _=tk.Event) -> None:
      self.select(None)
      self.notify("New task created")

    def refreshAll(self) -> None:
      self.calendar.updateCalendar(self.db.get_open_tasks())
      self.refreshTasks()

    def notify(self, msg: str) -> None:
      self.messageLabel.config(text=msg)
      print(msg)

    ######################################################
    # Task functions

    #Deletes the task selected in the listbox from the database
    def deleteTask(self, task: Task) -> None:
      if(tk.messagebox.askyesno(
          title="Confirm deletion",
          message=f"Are you sure you want to delete '{task.task_name}'?")):
        self.db.delete_task(task)
        self.notify(f"Deleted '{task.task_name}'")

        self.newTask()
        self.refreshAll()

    #Save the current state of the entry boxes for that task
    def save(self, task: Task) -> None:
        self.editingPane.timer.stop()

        if self.selection is None:
            selected = self.db.create_task(task)
        else:
            self.db.update_task(task)
            selected = task

        # This prevent the "do you want to save first? prompt from appearing"
        self.editingPane.selection = None
        #Refresh the screen
        self.refreshAll()
        self.select(selected) # TODO this doesn't highlight the newly-created task when creating a new task

        self.notify("Task saved")

    def select(self,  task: Optional[Task]) -> None:
        if self.editingPane.tryShow(task):
            self.selection = task
            self.scroller.highlightTask(task)
        else:
            self.notify("Cancelled")
