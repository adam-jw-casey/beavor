import tkinter as tk
import tkinter.messagebox
import tkinter.ttk as ttk
import datetime
from typing import Optional

from ..backend import parse_date, PyDueDate, Task
from .Timer import Timer
from .DateEntry import DateEntry
from .CompletingComboBox import CompletingComboBox
from .SensibleReturnWidget import SensibleReturnWidget, EntrySR, LabelSR, TextSR, FrameSR, CheckbuttonSR, ButtonSR

class EditingPane(tk.LabelFrame, SensibleReturnWidget):
    def __init__(self, parent, getSelectedTask, save, notify, get_categories, newTask, deleteTask, getDefaultTask):
        def canBeInt(_d, _i, _P, _s, S, _v, _V, _W) ->  bool:
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
            self,
            padx=8
        ).grid(row=0, column=0, sticky=tk.S+tk.N+tk.E+tk.W)
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(0, weight=1)

        # For save button, etc. below entry boxes
        self.entryButtonFrame = FrameSR(
            self
        ).grid(row=1, column=0, sticky=tk.S, pady=4)

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
        ).grid(sticky=tk.W, row=0, column=0, pady=1)
        self.categoryBox = CompletingComboBox(
            self.editing_box_frame,
            get_categories
        ).grid(sticky=tk.W+tk.E, row=0, column=1, pady=1)

        self.taskNameLabel = LabelSR(
            self.editing_box_frame,
            text="Task Name"
        ).grid(sticky=tk.W, row=1, column=0, pady=1)
        self.taskNameBox = EntrySR(
            self.editing_box_frame
        ).grid(sticky=tk.W+tk.E, row=1, column=1, pady=1)

        self.timeLabel = LabelSR(
            self.editing_box_frame,
            text="Time Needed"
        ).grid(sticky=tk.W, row=2, column=0, pady=1)
        self.timeBox = EntrySR(
            self.editing_box_frame,
            validate="key",
            validatecommand=int_validation
        ).grid(sticky=tk.W, row=2, column=1, pady=1)

        self.usedLabel = LabelSR(
            self.editing_box_frame,
            text="Time Used"
        ).grid(sticky=tk.W, row=3, column=0, pady=1)
        self.usedBox = EntrySR(
            self.editing_box_frame,
            validate="key",
            validatecommand=int_validation
        ).grid(sticky=tk.W, row=3, column=1, pady=1)

        self.nextActionLabel = LabelSR(
            self.editing_box_frame,
            text="Next Action"
        ).grid(sticky=tk.W, row=4, column=0, pady=1)
        self.nextActionBox = DateEntry(
            self.editing_box_frame,
            notify
        ).grid(sticky=tk.W, row=4, column=1, pady=1)

        self.dueDateLabel = LabelSR(
            self.editing_box_frame,
            text="Due Date"
        ).grid(sticky=tk.W, row=5, column=0, pady=1)
        self.dueDateBox = DateEntry(
            self.editing_box_frame,
            notify
        ).grid(sticky=tk.W, row=5, column=1, pady=1)

        self.notesLabel = LabelSR(
            self.editing_box_frame,
            text="Notes"
        ).grid(sticky=tk.W, row=6, column=0, pady=1)
        self.notesBox = TextSR(
            self.editing_box_frame,
            wrap="word"
        ).grid(sticky=tk.W+tk.E+tk.S+tk.N, row=6, column=1, pady=(1,4))
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
                match self._askSaveChanges(self.selection.name):
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
        self._overwriteEntryBox(self.taskNameBox,     task.name)
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

        task: Task = self.selection or self.getDefaultTask()

        try:
            task.category          = self.categoryBox.get()
            task.name              = self.taskNameBox.get()
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
                t1.name             != t2.name        or
                t1.time_needed      != t2.time_needed      or
                t1.time_used        != t2.time_used        or
                t1.next_action_date != t2.next_action_date or
                t1.notes            != t2.notes            or
                t1.due_date         != t2.due_date
            )

    def _clearEntryBoxes(self) -> None:
        self.doneIsChecked.set("O")
        self.timer.setTime(datetime.timedelta(0))
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
