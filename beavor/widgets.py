#!/usr/bin/python3.11

import tkinter as tk
import tkinter.messagebox
import tkinter.ttk as ttk
import datetime
import sys
from typing import List, Any, Optional

from .backend import green_red_scale, DatabaseManager, Task, PyDueDate, today_date, parse_date
from .ScrollFrame import ScrollFrame
from .Timer import Timer
from .CompletingComboBox import CompletingComboBox
from .DateEntry import DateEntry
from .SensibleReturnWidget import SensibleReturnWidget, EntrySR, LabelSR, TextSR, FrameSR, CheckbuttonSR, ButtonSR

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

        self.task_list_scroller = TaskScroller(
            self.root,
            self.select
        ).grid(row=0, column=0, pady=4, padx=4, sticky=tk.N+tk.S+tk.E+tk.W)
        self.root.grid_columnconfigure(0, weight=1)

        # Editing interface
        self.editingPane = EditingPane(
            self.root,
            self.getSelectedTask,
            self.save,
            self.notify,
            self.db.get_categories,
            self.newTask,
            self.deleteTask,
            self.db.default_task
        ).grid(row=0, column=1, padx=4, pady=4, sticky=tk.N+tk.S+tk.E+tk.W)
        self.root.grid_columnconfigure(1, weight=5)

        self.loadedTasks: List[Task] = []
        self.select(None)

        # Calendar
        self.calendar = Calendar(
            self.root,
            self.font
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

    def refreshTasks(self) -> None:
        #Remember which task was selected
        selected_rowid = self.selection.id if self.selection is not None else None

        self.loadedTasks = self.db.get_open_tasks()
        self.task_list_scroller.showTasks(self.loadedTasks)

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
            self.task_list_scroller.highlightTask(task)
        else:
            self.notify("Cancelled")

class EditingPane(tk.LabelFrame, SensibleReturnWidget):
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
        self.editing_box_frame = FrameSR(
            self
        ).grid(row=0, column=0, sticky=tk.S+tk.N+tk.E+tk.W)
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(0, weight=1)

        # For save button, etc. below entry boxes
        self.entryButtonFrame = FrameSR(
            self
        ).grid(row=1, column=0, sticky=tk.S)

        # Timer and its button
        self.timer = Timer(
            self.entryButtonFrame,
            getSelectedTask,
            self.save,
            lambda time: self._overwriteEntryBox(self.usedBox, time),
            notify
       ).grid(sticky=tk.S, row=0, column=1)

        #Setup the lower half of the window
        self.categoryLabel = LabelSR(
            self.editing_box_frame,
            text= "Category"
        ).grid(sticky=tk.W, row=0, column=0)
        self.categoryBox = CompletingComboBox(
            self.editing_box_frame,
            get_categories
        ).grid(sticky=tk.W+tk.E, row=0, column=1)

        self.taskNameLabel = LabelSR(
            self.editing_box_frame,
            text="Task Name"
        ).grid(sticky=tk.W, row=1, column=0)
        self.taskNameBox = EntrySR(
            self.editing_box_frame
        ).grid(sticky=tk.W+tk.E, row=1, column=1)

        self.timeLabel = LabelSR(
            self.editing_box_frame,
            text="Time Needed"
        ).grid(sticky=tk.W, row=2, column=0)
        self.timeBox = EntrySR(
            self.editing_box_frame,
            validate="key",
            validatecommand=int_validation
        ).grid(sticky=tk.W, row=2, column=1)

        self.usedLabel = LabelSR(
            self.editing_box_frame,
            text="Time Used"
        ).grid(sticky=tk.W, row=3, column=0)
        self.usedBox = EntrySR(
            self.editing_box_frame,
            validate="key",
            validatecommand=int_validation
        ).grid(sticky=tk.W, row=3, column=1)

        self.nextActionLabel = LabelSR(
            self.editing_box_frame,
            text="Next Action"
        ).grid(sticky=tk.W, row=4, column=0)
        self.nextActionBox = DateEntry(
            self.editing_box_frame,
            notify
        ).grid(sticky=tk.W, row=4, column=1)

        self.dueDateLabel = LabelSR(
            self.editing_box_frame,
            text="Due Date"
        ).grid(sticky=tk.W, row=5, column=0)
        self.dueDateBox = DateEntry(
            self.editing_box_frame,
            notify
        ).grid(sticky=tk.W, row=5, column=1)

        self.notesLabel = LabelSR(
            self.editing_box_frame,
            text="Notes"
        ).grid(sticky=tk.W, row=6, column=0)
        self.notesBox = TextSR(
            self.editing_box_frame,
            wrap="word"
        ).grid(sticky=tk.W+tk.E+tk.S+tk.N, row=6, column=1, pady=(0,4))
        self.editing_box_frame.grid_rowconfigure(6, weight=1)

        self.editing_box_frame.grid_columnconfigure(1, weight=1)

        self.doneIsChecked = tk.StringVar()
        self.doneCheckBox = CheckbuttonSR(
            self.entryButtonFrame,
            text="Done",
            variable=self.doneIsChecked,
            onvalue="X",
            offvalue="O"
        ).grid(row=0, column=0)
        self.doneCheckBox.deselect()

        #Add buttons to interact
        self.saveButton = ButtonSR(
            self.entryButtonFrame,
            text="Save",
            command=self.save
        ).grid(row=0, column=2)

        self.newTaskButton = ButtonSR(
            self.entryButtonFrame,
            text="New",
            command=newTask
        ).grid(row=0, column=3)

        self.deleteButton = ButtonSR(
            self.entryButtonFrame,
            text="Delete",
            command = lambda: deleteTask(self.selection)
        ).grid(row=0, column=4)

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

class TaskScroller(ScrollFrame, SensibleReturnWidget):
    def __init__(self, parent: tk.Frame | tk.LabelFrame, onRowClick):
        super().__init__(parent, "Tasks")
        self.taskRows: list[TaskRow] = []
        self.tasks: list[Task] = []
        self.onRowClick = onRowClick

        self.show_available_only = tk.BooleanVar()
        self.show_available_only.set(True)

        self.available_only_button = CheckbuttonSR(
            self,
            text="Show only available tasks",
            variable=self.show_available_only,
            command = lambda: self.showTasks(self.tasks)
        ).grid(row=1, column=0, sticky=tk.E+tk.W)

    def showTasks(self, tasks: list[Task]) -> None:
        self.tasks = tasks

        for _ in range(len(self.taskRows)):
            self.taskRows.pop().destroy()
            
        for (i, task) in enumerate(filter(lambda t: (not self.show_available_only.get()) or t.next_action_date <= today_date(), tasks)):
            taskRow = TaskRow(
                self.viewPort,
                task,
                lambda t=task: self.onRowClick(t)
            ).grid(row=i, column=0, sticky= tk.W+tk.E)
            self.taskRows.append(taskRow)

    def highlightTask(self, task: Optional[Task]) -> None:
        for tr in self.taskRows:
            if task is not None and tr.id == task.id:
                tr.highlight()
            else:
                tr.unhighlight()

class TaskRow(tk.LabelFrame, SensibleReturnWidget):
    def __init__(self, parentFrame: tk.Frame, task, select):
        super().__init__(parentFrame)
        self.select = select
        self.id = task.id

        self.nameLabel = LabelSR(
            self,
            text=task.task_name
        ).grid(row=0, column=0, sticky = tk.W + tk.E)

        self.categoryLabel = LabelSR(
            self,
            text=task.category,
            font=("helvetica",
            8)
        ).grid(row=1, column=0, sticky=tk.W)

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
