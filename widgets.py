#!/usr/bin/python3.11

import sqlite3
import tkinter as tk
import tkinter.messagebox
import tkinter.ttk as ttk
import datetime
import time
import sys
import numpy as np
import re
from dateutil.relativedelta import relativedelta
import platform
from typing import List, Any
from enum import Enum

from utils import *

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
# todo user-customizable settings (like font size, calendar colourscale) -> This could write to external file read at startup?
# todo Dark mode toggle (use .configure(bg='black') maybe? Or another better colour. Have to do it individually by pane though, self.root.configure() only does some of the background. Also probably have to change text colour too.)
# todo User-adjustable font/font size

###########################################

class Timer(tk.Frame):

    TIME_FMT = "%H:%M:%S"

    def __init__(self, parent: tk.Frame | tk.LabelFrame, font, getSelectedTask, save, setUsedTime):
        super().__init__(parent)

        self.timeLabel = tk.Label(self, text="0:00:00", font=font)
        self.timeLabel.grid(row=0, column=1)

        self.timeButton = tk.Button(self, text="Start", command=lambda: self.toggleTimer(getSelectedTask()))
        self.timeButton.grid(row=0, column=0)
        self.timeButton.bind("<Return>", lambda _: self.toggleTimer(getSelectedTask()))
        self.timing = False

        self.save = save
        self.setUsedTime = setUsedTime

    def toggleTimer(self, selected_task) -> None:
      if not self.timing:
        self.start(selected_task)
      else:
        self.save()

    def start(self, task) -> None:
      if task is None:
        # todo better way of handling this -> this dumps the exception in the console
        raise self.EmptyTaskError("Cannot time an empty task")

      self.timeButton.config(text="Stop")
      self.startTime = time.strftime(self.TIME_FMT)
      self.initialTime = datetime.timedelta(minutes=(task["Used"] or 0))

      self.timing = True
      self._update_diplayed_time()

    def stop(self):
        self.timeButton.config(text="Start")
        self.timing = False
        self.setUsedTime(round(self.timerVal.total_seconds()/60))

    def setDisplay(self, time):
        self.timeLabel.config(text=str(time))

    def askSaveTimer(self, taskName: str) -> bool:
        return tk.messagebox.askyesnocancel(title="Save before switching?",
                                            message=f"Do you want to save the timer for '{taskName}' before switching?")

    def _update_diplayed_time(self):
        if self.timing:
            # TODO would be better to handle this by accounting for date rather than just fudging the days
            runTime = (datetime.datetime.strptime(time.strftime(self.TIME_FMT), self.TIME_FMT)
                       - datetime.datetime.strptime(self.startTime, self.TIME_FMT))
            # If the timer is run through midnight it goes negative. This fixes it.
            if runTime.days < 0:
              runTime = runTime + datetime.timedelta(days=1)

            self.timerVal = (runTime + self.initialTime)
            self.setDisplay(self.timerVal)

            self.after(1000, self._update_diplayed_time)

    class EmptyTaskError(Exception):
      pass

class DateEntry(tk.Entry):
  def __init__(self, parentFrame, notificationFunc):
    super().__init__(parentFrame)
    self.notify = notificationFunc
    self.bind("<Tab>", self.convertDate)

  def convertDate(self, event=tk.Event) -> None:
    box = event.widget
    dateStr = box.get()
    convertedDate = ""

    try:
      YMDstr2date(dateStr)
      convertedDate = dateStr
    except ValueError:
      try:
        #eg. Jan 1, 21
        convertedDate = date2YMDstr(datetime.datetime.strptime(dateStr, "%b %d, %y"))
      except ValueError:
        #Date string doesn't match
        try:
          #Try to add the current year
          #eg. Jan 1
          convertedDate = date2YMDstr(datetime.datetime.strptime(dateStr, "%b %d").replace(year = todayDate().year))
        except ValueError:
          #Date really doesn't match
          self.notify("Can't match date format of {}".format(dateStr))
          return

    box.delete(0, tk.END)
    box.insert(0, convertedDate)

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

    #Scale all padding by this multiplier (not tested lol)
    self.padscale = 1

    # Frame to hold the tasklist display and associated frames and widgets
    self.taskDisplayFrame = tk.LabelFrame(self.root, text="Tasks", padx=self.padscale*4, pady=self.padscale*4)
    self.taskDisplayFrame.grid(row=0, column=0, pady=self.padscale * 4, padx=self.padscale * 4, sticky=tk.N+tk.S+tk.E+tk.W)

    # Frame for all the buttons and boxes below the tasklist pane
    self.interactiveFrame = tk.Frame(self.root)
    self.interactiveFrame.grid(row=0, column=1, pady=self.padscale * 4)

    # Frame for the calendar
    self.calendarFrame = Calendar(self.interactiveFrame, self.font)
    self.calendarFrame.grid(row=0, column=3, pady=self.padscale * 4, padx=self.padscale * 4, sticky=tk.S)

    # Entry boxes and labels
    self.entryFrame = tk.Frame(self.interactiveFrame)
    self.entryFrame.grid(row=0, column=1)

    # For save button, etc. below entry boxes
    self.entryButtonFrame = tk.Frame(self.interactiveFrame)
    self.entryButtonFrame.grid(row=1, column=1)

    self.selection = None

    # Timer and its button
    self.timer = Timer(self.entryButtonFrame, self.font, self.getSelectedTask, self.save, lambda time: self.overwriteEntryBox(self.entryBoxes["Used"], time))
    self.timer.grid(row=0, column=1, padx=self.padscale * (0,30))

    self.loadTasks()

    recordLabel = tk.Label(self.taskDisplayFrame, text="")
    self.scroller = TaskScroller(self.taskDisplayFrame, self.selectTask, recordLabel)
    self.scroller.pack(side=tk.TOP, fill="both", expand=True)
    recordLabel.pack(side=tk.BOTTOM)
    recordLabel.pack(side=tk.RIGHT)

    #Setup the lower half of the window

    self.editColumns = ["Category", "Task", "Time", "Used", "NextAction", "DueDate", "Flex", "Notes"]

    self.entryBoxes: dict[str, ttk.Combobox | tk.Text | tk.Entry | DateEntry] = {}
    self.entryLabels: dict[str, tk.Label] = {}

    #Add inputs below list
    for i, header in enumerate(self.editColumns):
      self.entryLabels[header] = tk.Label(self.entryFrame, text=header)
      self.entryLabels[header].grid(sticky="W",row=i, column=0)

      if header == "Category":
        self.entryBoxes[header] = ttk.Combobox(self.entryFrame)
        self.entryBoxes[header].bind("<FocusOut>", self.clearComboHighlights)
        self.entryBoxes[header].bind("<KeyRelease>", lambda event: self.completeBox(event, self.categories))
        self.entryBoxes[header].bind("<Return>", lambda _: self.entryBoxes["Category"].icursor(tk.END))
      elif header == "Flex":
        self.entryBoxes[header] = ttk.Combobox(self.entryFrame,
                                                  values=["Y","N"],
                                                  state="readonly")
        self.entryBoxes[header].bind("<FocusOut>", self.clearComboHighlights)
      elif header == "Notes":
        self.entryBoxes[header] = tk.Text(self.entryFrame, wrap="word")
        self.entryBoxes[header].bind("<Tab>", self.focusNextWidget)
        self.entryFrame.grid_columnconfigure(i, weight=1)
      else:
        if header in ["DueDate", "NextAction"]:
          self.entryBoxes[header] = DateEntry(self.entryFrame, self.notify)
        else:
          self.entryBoxes[header] = tk.Entry(self.entryFrame)
        self.entryBoxes[header].bind("<Return>", self.save)

      self.entryBoxes[header].grid(sticky="NW",row=i, column=1, pady=self.padscale * 1)
      self.entryBoxes[header].config(width=50, font=self.font)

    self.checkDone = tk.StringVar()
    self.doneCheck = tk.Checkbutton(self.entryButtonFrame,
                                    text="Done",
                                    variable=self.checkDone,
                                    onvalue="X",
                                    offvalue="O")
    self.doneCheck.grid(row=0, column=0)
    self.doneCheck.deselect()

    #Add buttons to interact
    self.saveButton = tk.Button(self.entryButtonFrame, text="Save", command=self.save)
    self.saveButton.grid(row=0, column=2)
    self.saveButton.bind("<Return>", self.save)

    self.newTaskButton = tk.Button(self.entryButtonFrame, text="New", command = self.newTask)
    self.newTaskButton.grid(row=0, column=3)
    self.newTaskButton.bind("<Return>", self.newTask)

    self.deleteButton = tk.Button(self.entryButtonFrame,
                                  text="Delete",
                                  command = self.deleteTask)
    self.deleteButton.grid(row=0, column=4)
    self.deleteButton.bind("<Return>", self.deleteTask)

    self.duplicateButton = tk.Button(self.entryButtonFrame,
                                     text="Duplicate",
                                     command=self.duplicateTask)
    self.duplicateButton.grid(row=0, column=5)

    self.messageLabel = tk.Label(self.interactiveFrame, text="")
    self.messageLabel.grid(column=0, columnspan=3)

    # These aren't all the keybindings, but they're all the ones the user should notice
    # Other keybindings mostly just make the app behave how you'd expect
    self.root.bind("<Control-q>", lambda _: self.root.destroy())
    self.root.bind("<Control-w>", lambda _: self.root.destroy())
    self.root.bind("<Control-n>", lambda _: self.newTaskButton.invoke())

    self.refreshAll()

  ######################################################
  # GUI update functions

  #Clears the annoying highlighting from all comboboxes
  def clearComboHighlights(self, _=tk.Event) -> None:
    for header in ["Category", "Flex"]:
      self.entryBoxes[header].selection_clear()


  def completeBox(self, event: tk.Event, sourceList: List[str]) -> None:
    #Don't run when deleting, or when shift is released
    if event.keysym not in ["BackSpace", "Shift_L", "Shift_R"]:
      box = event.widget
      cursorPos:int = box.index(tk.INSERT)
      current: str = box.get()[0:cursorPos]
      #Don't run if box is empty, or cursor is not at the end
      if current and cursorPos == len(box.get()):
        options = []
        for item in sourceList:
          #If any categories begin with the current string up to the cursor
          if item.find(current) == 0:
            options.append(item)

        if options:
          try:
            #Finds index of last character in common
            i = len(current)
            while len(set([option[i] for option in options])) == 1:
              i+=1
          except IndexError:
              #todo wtf
            #Iterating out the end of one of the options
            pass

          i-=1

          #only if found text longer than current in common
          if i > len(current) - 1:
            box.insert(tk.END, options[0][cursorPos:i+1])

          box.select_range(cursorPos, tk.END)
          box.icursor(tk.END)

  def getSearchCriteria(self) -> list[str]:
    return ["O != 'X'", "NextAction <= '{}'".format(todayStr())]

  def refreshTasks(self, _=tk.Event) -> None:
    #Remember which task was selected
    if self.selection != None:
      self.selected_rowid = self.selection["rowid"]

    criteria = self.getSearchCriteria()
    self.loadTasks(criteria)
    self.scroller.showTasks(self.loadedTasks)

    if self.selection != None:
      previousSelection = None
      for i, task in enumerate(self.loadedTasks):
        if task["rowid"] == self.selected_rowid:
          previousSelection = i
          break

      if previousSelection is not None:
        self.selection = self.loadedTasks[previousSelection]
        self.scroller.unhighlightAndSelectTask(self.selection)
      else:
        self.clearEntryBoxes()

  def askSaveChanges(self, taskName: str) -> bool:
    if self.nonTrivialChanges():
      return tk.messagebox.askyesnocancel(title="Save before switching?",
                                          message=f"Do you want to save your changes to '{taskName}' before switching?")
    else:
        return False

  def newTask(self, _=tk.Event) -> None:
    self.clearEntryBoxes()
    self.notify("Creating new entry")
    self.scroller.unhighlightAndSelectTask(None)

  #Bound to the Tab key for Text box, so that it will cycle widgets instead of inserting a tab character
  def focusNextWidget(self, event: tk.Event) -> str:
    event.widget.tk_focusNext().focus()
    return("break")

  def clearEntryBoxes(self) -> None:
    if self.selection is not None:
        # TODO shouldn't have to check this here - should probably be done higher up????
        match self.askSaveChanges(self.selection["Task"]):
            case True:
                self.save()
            case False:
                pass
            case None:
                # TODO this is janky - fix it
                raise PermissionError("Cancelled by user")

        #Nothing selected, just clear the box
        self.checkDone.set("O")
        self.entryBoxes["Flex"].set("")
        self.timer.setDisplay("0:00:00")
        for header in self.editColumns:
          self.overwriteEntryBox(self.entryBoxes[header], "")

  def overwriteEntryBox(self, entry: ttk.Combobox | tk.Text | tk.Entry | DateEntry, text) -> None:
    #Check if we need to temporarily enable the box
    changeFlag = (entry["state"] == "readonly")
    if changeFlag:
      entry.config(state="normal")

    try:
      entry.delete('1.0','end')# tk.text
    except tk.TclError:
      entry.delete(0,'end')# tk.Entry
    entry.insert('end', text)

    #Switch back to the original state
    if changeFlag:
      entry.config(state=tk.DISABLED)

  def selectTask(self, task) -> None:
    self.messageLabel.config(text="")
    if self.timer.timing:
        match self.timer.askSaveTimer(task["Task"]):
            case True:
                self.timer.save()
            case False:
                self.timer.stop()
            case None:
                return

    if task is None:
       self.clearEntryBoxes()
    else:
        match self.askSaveChanges(task["Task"]):
            case True:
                self.save()
            case False:
                pass
            case None:
                return

        #todo this could be a function "update entryBoxes" or something
        for (header, entry) in [(header, self.entryBoxes[header]) for header in self.editColumns]:
          if header == "Flex":
            entry.set(task[header])
          else:
            self.overwriteEntryBox(entry, task[header])

        self.checkDone.set(task["O"])

        self.selection = task

        if not self.timer.timing:
          self.timer.setDisplay(datetime.timedelta(minutes=(task["Used"] or 0)))

  def refreshAll(self, _=tk.Event) -> None:
    self.refreshCategories()
    self.updateLoadsToday()
    self.calendarFrame.updateCalendar(self.db.getTasks4Workload())
    self.refreshTasks()

  def refreshCategories(self) -> None:
    self.categories = self.db.getCategories()
    try:
      self.entryBoxes["Category"].config(values=self.categories)
    except AttributeError:
      #Fails on setup
      pass


  def notify(self, msg: str) -> None:
    try:
      self.messageLabel.config(text=msg)
    except AttributeError:
      # Fails on startup
      pass
    print(msg)

  ######################################################
  # Calculation functions

  #Gets the text in the passed entryBox
  def getEntry(self, entryBox: tk.Text | ttk.Combobox | tk.Entry | DateEntry) -> str:
    try:
      return entryBox.get()
    except TypeError:
      #Notes
      return entryBox.get('1.0','end')[:-1]

  #validate data
  def validateRow(self, row: dict | sqlite3.Row) -> None:
      for header in self.db.headers:
        data = row[header]
        if header in ["NextAction", "DueDate", "DateAdded"]:
          try:
              YMDstr2date(data)
          except ValueError:
            if not (header == "DateAdded" and data == ""):
              raise ValueError("Incorrect date format: {}, {} should be YYYY-MM-DD".format(header, data))
        elif "Time" in header:
          #Checks that times are numbers, and positive
          try:
            if int(data) < 0:
              raise ValueError
          except ValueError:
            if data != "":
              raise ValueError("{} must be >= 0, not '{}'".format(header, data))
        elif header == "Flex" and data not in ["Y","N"]:
          raise ValueError("{} should be Y or N, not '{}'".format(header, data))
        elif header in ["Task", "Notes"] and type(data) != type(str()):
          raise ValueError("Unacceptable {}: {}".format(header, data))

  # takes a dict (or sqlite3.Row) representing a task, updates all calculated values and returns the new Row
  def calculateRow(self, inRow: dict | sqlite3.Row, _=tk.Event) -> dict:
    today = todayStr()

    newRowDict: dict = {}
    for header in self.db.headers:
      if header == "Left":
        newRowDict[header] = max(0, int(newRowDict["Time"] or 0) - int(newRowDict["Used"] or 0))
      elif header == "DaysLeft":
        newRowDict[header] = workDaysBetween(today, newRowDict["DueDate"])
      elif header == "TotalLoad":
        if inRow["O"] == "O":
          newRowDict[header] = round((1.1 if inRow["Flex"] == "N" else
              1)*newRowDict["Left"]/max(1,(newRowDict["DaysLeft"] if
                  newRowDict["NextAction"] <= today else workDaysBetween(newRowDict["NextAction"], newRowDict["DueDate"]))),1)
        else:
          newRowDict[header] = None
      elif header == "Load":
        newRowDict[header] = (newRowDict["TotalLoad"] if newRowDict["NextAction"] <= today else None)
      else:
        newRowDict[header] = inRow[header]

    return newRowDict

  ######################################################
  # Task functions

  # Wrapper for the database manager equivalent function
  def loadTasks(self, criteria: list[str] = []) -> None:
    self.loadedTasks = self.db.getTasks(criteria)
    if len(self.loadedTasks) == 0:
      self.notify("No tasks found")

  #Deletes the task selected in the listbox from the database
  def deleteTask(self) -> None:
    task = self.selection
    try:
      deleted = False
      if(tk.messagebox.askyesno(title="Confirm deletion",
                                message="Are you sure you want to delete '{}'?".format(
                                          task["Task"]))):
        self.db.deleteByRowid(task["rowid"])
        self.notify("Deleted '{}'".format(task["Task"]))
        deleted = True

      # Only need to do this if deleted a task
      if deleted:
        self.db.commit()

        self.clearEntryBoxes()
        self.refreshAll()

        self.newTask()
        self.refreshTasks()

    except TypeError:
      self.notify("Cannot delete - none selected")

  #Save the current state of the entry boxes for that task
  def save(self, _=tk.Event()) -> None:
    if self.timer.timing:
        self.timer.stop()

    try:
      if self.selection is None:
        self.createTaskFromInputs()
      else:
        self.updateSelectedTask()

      #Refresh the screen
      self.refreshAll()

      self.notify("Task saved")
    except ValueError as e:
      #Something wrong with the inputs given
      self.notify(str(e))
    except PermissionError as e:
      #updateSelectedTask() cancelled by user
      self.notify(str(e))

  # TODO a more elegant way of handling repeating tasks than just creating a bunch of duplicates. Maybe a task that duplicates itself a number of days in the future when completed?
  def createTaskFromInputs(self) -> None:
    newRowDict = {}

    #Pull in directly entered values
    for header in self.editColumns:
      newRowDict[header] = self.getEntry(self.entryBoxes[header])

    #Store original values
    newRowDict["Budget"] = newRowDict["Time"]
    newRowDict["StartDate"] = newRowDict["NextAction"]
    newRowDict["DateAdded"] = todayStr()
    #Defaults
    newRowDict["O"] = "O"
    newRowDict["rowid"] = None

    #Creating single task
    # 1 task, with no offset
    repetitions = 1
    interval = relativedelta()

    # Iterate over tasks to create. For single task creation, runs only once
    for i in range(repetitions):
      thisRowDict = newRowDict.copy()
      for header in ["StartDate", "NextAction", "DueDate"]:
        thisRowDict[header] = date2YMDstr(YMDstr2date(thisRowDict[header]) + i * interval)

      thisRowDict = self.calculateRow(thisRowDict)
      self.validateRow(thisRowDict)

      headers = [h for h in self.db.headers if h != "rowid"]
      vals = [thisRowDict[header] for header in headers]

      self.db.createTask(headers, vals)

    self.db.commit()
    # This is so you don't accidentally create multiple of the same task by clicking save multiple times
    self.clearEntryBoxes()

  # Like updateSelectedTask, but you pass the updated task in rather than pulling from input
  # Doesn't commit the changes, so you can loop without a huge overhead
  def updatePassedTask(self, row: dict | sqlite3.Row) -> None:
    #Find which columns were changed and how
    changes = []
    newRow = {}

    self.validateRow(row)
    newRow = self.calculateRow(row)
    for header in self.db.headers:
      changes.append(" {} = '{}' ".format(header, escapeSingleQuotes(str(newRow[header]))))

    criteria = ["rowid = {}".format(row["rowid"])]

    self.db.updateTasks(criteria, changes)

  def nonTrivialChanges(self) -> bool:
    changes = self.getChanges()
    if len(changes) == 1 and "DaysLeft" in changes[0]:
      return False
    elif len(changes) == 2 and "DaysLeft" in changes[0] and "TotalLoad" in changes[1]:
      return False
    elif len(changes) == 0:
      return False
    else:
      return True

  def getChanges(self) -> list[str]:
    changes = []
    if self.selection == None:
      pass
    else:
      #Find which columns were changed and how

      newRow = {}
      oldRow = dict(self.selection)

      for (header, old) in [(header, oldRow[header]) for header in self.db.headers]:
        if header in self.editColumns + ["O"]:
          #This is a checkbox and not in the edit list
          if header == "O":
            # double ifs so "O" can't fall through
            new = self.checkDone.get()
          else:
            new = self.getEntry(self.entryBoxes[header])

            try:
              new = type(old)(new)
            except ValueError as e:
              if type(old) == int and new == '':
                new = 0
              else:
                raise ValueError("Bad input: {}".format(e))

          newRow[header] = new
        else:
          newRow[header] = old

      self.validateRow(newRow)
      newRow = self.calculateRow(newRow)

      for header in self.db.headers:
        new = newRow[header]

        if new == None:
          new = "None"

        old = oldRow[header]
        if new != old:
          if header == "DueDate" or header == "NextAction":
            dateChange = daysBetween(old, new)
            changes.append(" {} = date({}, '{} days') ".format(header, header, dateChange))
          elif header == "Used":
              # For time tracking
              timediff = int(new if new != '' else 0) - int(old if old != '' else 0)
              changes.append(" {} = {} + {} ".format(header, header, timediff))
          else:
              changes.append(" {} = '{}' ".format(header, escapeSingleQuotes(str(new))))

    return changes

  # Update the currently selected task with values from the entry boxes
  def updateSelectedTask(self) -> None:
    changes = self.getChanges()

    if changes:
      criteria = ["rowid = {}".format(self.selection["rowid"])]

      # todo messy
      # Dump the time worked to external time tracker
      for change in changes:
          if change.find("Used") != -1:
              timediff = int(re.findall(r"(\d+)", change)[0])
              with open("timesheet.csv", "a") as f:
                  f.write("{}, {}, {}, {}\n".format(todayStr(), self.selection["Category"], timediff, self.selection["Task"]))

      self.db.updateTasks(criteria, changes)

      self.db.commit()

  def duplicateTask(self) -> None:
    oldSelection = self.selection
    self.selection = None
    self.save()
    self.scroller.unhighlightAndSelectTask(oldSelection)
    self.notify("Duplicated task")

  #scans all tasks and updates using calculateRow()
  def updateLoadsToday(self, _=tk.Event) -> None:
    #backup task list
    oldTasks = self.loadedTasks

    self.loadTasks(["O == 'O'","NextAction <= '{}'".format(todayStr())])

    #Don't commit until the end - saves a few seconds each time
    for task in self.loadedTasks:
      self.updatePassedTask(task)
    self.db.commit()

    #put original task list back
    self.loadedTasks = oldTasks

# todo put the next action / due date at a specific time?
# todo add buttons to scroll the calendar forward week-by-week
# todo Days of the week shown should be user-configurable (M-F vs. student schedule lol, or freelance).
# eg. thisDay["LoadLabel"].bind("<Button-1>", CALLBACK)
# Set up the calendar display to show estimated workload each day for a several week forecast
class Calendar(tk.LabelFrame):
  def __init__(self, parentFrame: tk.Frame , parentFont):
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

  def updateCalendar(self, openTasks) -> None:

    self.calculateDayLoads(openTasks)

    today = todayDate()
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
          hoursThisDay = self.getDayTotalLoad(date2YMDstr(thisDate)) / 60
          thisDay["LoadLabel"].config(text=str(round(hoursThisDay,1)),
                                      bg=greenRedScale(0,(7 if thisDate != today else max(0.1, hoursLeftToday)), hoursThisDay))
        else:
          thisDay["LoadLabel"].config(text="", bg="#d9d9d9")

  def calculateDayLoads(self, openTasks) -> None:
    # Get a list of all unfinished tasks with start dates no more than self.numweeks in the future, sorted from soonest due date to latest
    today = todayDate()
    thisFriday = today - datetime.timedelta(days=today.weekday() + 4)
    lastRenderedDate = thisFriday + datetime.timedelta(weeks=self.numweeks-1)
    endDate = date2YMDstr(lastRenderedDate)
    relevantTasks = [task for task in openTasks if task["NextAction"] <= endDate]

    # Iterate over the list of tasks (starting from soonest due date), distributing time evenly (each day gets time remaining / # days remaining) over days from max(today, start date) to due date. If adding time would push day over 8 hours, only add up to 8 hours, and withold extra time within the task.
    self.dayLoads: dict[str, float] = {}
    # TODO this code ignores work start time and lunch break, i.e. at midnight it will assume there are 16 hours of work left today, and at 7 AM it will assume there are 9
    # todo another way to do this would be to save how many hours of work are due today and to subtract the number of hours of tracked work. That would avoid rewarding with less work remaining simply because time has passed.
    # calculates the time (in hours) remaining until 4 PM system time, because I work an hour ahead (i.e. 4 PM system time is 5 PM my time)
    hoursLeftToday = max(0, 16 - (datetime.datetime.now().hour + datetime.datetime.now().minute/60))
    for task in relevantTasks:
      # todo around here would be a decent place to do recursion. A function like def distributeTimeOverRange(time, range)
      remainingLoad = task["Left"]
      startDate = max(today, YMDstr2date(task["NextAction"]))
      dateRange = [startDate + datetime.timedelta(days=n) for n in range(0, daysBetween(date2YMDstr(startDate), task["DueDate"]) + 1)]

      for thisDay in dateRange:
        maxHours = (6 if thisDay != today else hoursLeftToday)
        if np.is_busday(thisDay):
          # TODO This needs to change once the overflow code down below is fixed. This backloads time by squishing extra time away, rather than distributing evenly or optimally. Note that this does not OPTIMALLY backload time, it simply backloads relative to an even distribution.
          # TODO would also be better to individually count work days in the range. This assumes that no days are missing from the range (such as if they had been previously excluded because of being full)
          # todo would be nice if could switch between workload distribution modes - i.e.:
          #    - evenload: spread the work as evenly as possible over available days
          #      This is the system I've been using up to now, but with the added max work per day cap.
          #      evenload could be implemented as described in the TODOs above:
          #      allocate time evenly across available days. When even distribution would overwhelm a day, remove it from the list and continue at same rate. At the end, divide the remaining time and repeat over the range (with the overwhelmed day removed).
          #    - A frontload mode that pushes as much work to the front of availability as possible, without exceeding daily cap.
          #      This would be useful to show the amount of work available (when will I need to start looking for more projects?)
          #      This frontload mode is essentially what I have been working towards, as it is best at guaranteeing that tasks are completed.
          #      This could be implemented by, for each task, starting from soonest due, allocating as much time as possible to each day from first available, until task is complete, without exceeding daily max.
          #    - A backload mode to push work as far as possible while still completing all tasks.
          #      This would visualize how crunched I actually am.
          #      backload is useful because it lets me know whether my time this week is actually full, or if I could move things around to take on more work.
          #      This could be implemented by, for each task starting from soonest due, allocating as much time as possible to each day from last available, until task is complete, without exceeding daily max.
          loadDeposit = remainingLoad / workDaysBetween(thisDay, task["DueDate"])
          # Do not push a day over 8 hours
          try:
              loadDeposit = min(max(maxHours*60 - self.dayLoads[date2YMDstr(thisDay)], 0), loadDeposit)
              self.dayLoads[date2YMDstr(thisDay)] += loadDeposit
          except KeyError:
              # If this day has no load assigned to it yet, there will not be an entry in the dict and a key error will occur
              loadDeposit = min(maxHours*60, loadDeposit)
              self.dayLoads[date2YMDstr(thisDay)] = loadDeposit

          remainingLoad -= loadDeposit

        # TODO If time remains (i.e. one or more days was maxed out to 8 hours), distribute remaining time evenly over all tasks (TODO: doing it recursively, noting the number of days maxed out and using a new quotient to calculate average load each time would be better, although you would need an end condition other than "all time distributed" since it's not guaranteed that all days can be kept to 8 hours or less with this method).
          if date2YMDstr(thisDay) == task["DueDate"] and remainingLoad != 0:
            self.dayLoads[date2YMDstr(thisDay)] += remainingLoad
            remainingLoad = 0 # unecessary but comforts me

  # Gets the work load for the day represented by the passed string
  #date should be a string formatted "YYYY-MM-DD"
  def getDayTotalLoad(self, date: str) -> float:
    # Will raise an error if date is poorly formatted
    YMDstr2date(date)

    try:
      return self.dayLoads[date]
    except KeyError:
      return 0;

class ScrollFrame(tk.Frame):
    def __init__(self, parent: tk.Frame | tk.LabelFrame):
        super().__init__(parent) # create a frame (self)

        self.canvas = tk.Canvas(self, borderwidth=0)                                #place canvas on self
        self.viewPort = tk.Frame(self.canvas)                                       #place a frame on the canvas, this frame will hold the child widgets
        self.vsb = tk.Scrollbar(self, orient="vertical", command=self.canvas.yview) #place a scrollbar on self
        self.canvas.configure(yscrollcommand=self.vsb.set)                          #attach scrollbar action to scroll of canvas

        self.vsb.pack(side="right", fill="y")                                       #pack scrollbar to right of self
        self.canvas.pack(side="left", fill="both", expand=True)                     #pack canvas to left of self and expand to fil
        self.canvas_window = self.canvas.create_window((4,4), window=self.viewPort, anchor="nw",            #add view port frame to canvas
        tags="self.viewPort")

        self.viewPort.bind("<Configure>", self.onFrameConfigure)                    #bind an event whenever the size of the viewPort frame changes.
        self.canvas.bind("<Configure>", self.onCanvasConfigure)                     #bind an event whenever the size of the canvas frame changes.

        self.viewPort.bind('<Enter>', self.onEnter)                                 # bind wheel events when the cursor enters the control
        self.viewPort.bind('<Leave>', self.onLeave)                                 # unbind wheel events when the cursorl leaves the control

        self.onFrameConfigure(None)                                                 #perform an initial stretch on render, otherwise the scroll region has a tiny border until the first resize

    def onFrameConfigure(self, _):
        '''Reset the scroll region to encompass the inner frame'''
        self.canvas.configure(scrollregion=self.canvas.bbox("all"))                 #whenever the size of the frame changes, alter the scroll region respectively.

    def onCanvasConfigure(self, event: tk.Event):
        '''Reset the canvas window to encompass inner frame when required'''
        canvas_width = event.width
        self.canvas.itemconfig(self.canvas_window, width = canvas_width)            #whenever the size of the canvas changes alter the window region respectively.

    def onMouseWheel(self, event: tk.Event):                                                  # cross platform scroll wheel event
        if platform.system() == 'Windows':
            self.canvas.yview_scroll(int(-1* (event.delta/120)), "units")
        elif platform.system() == 'Darwin':
            self.canvas.yview_scroll(int(-1 * event.delta), "units")
        else:
            if event.num == 4:
                self.canvas.yview_scroll( -1, "units" )
            elif event.num == 5:
                self.canvas.yview_scroll( 1, "units" )

    def onEnter(self, _):                                                       # bind wheel events when the cursor enters the control
        if platform.system() == 'Linux':
            self.canvas.bind_all("<Button-4>", self.onMouseWheel)
            self.canvas.bind_all("<Button-5>", self.onMouseWheel)
        else:
            self.canvas.bind_all("<MouseWheel>", self.onMouseWheel)

    def onLeave(self, _):                                                       # unbind wheel events when the cursorl leaves the control
        if platform.system() == 'Linux':
            self.canvas.unbind_all("<Button-4>")
            self.canvas.unbind_all("<Button-5>")
        else:
            self.canvas.unbind_all("<MouseWheel>")

class TaskScroller(ScrollFrame):
    def __init__(self, parent: tk.Frame | tk.LabelFrame, selectTask, recordLabel):
        super().__init__(parent)
        self.tasks = []
        self.selectTask = selectTask

        self.recordLabel = recordLabel

    def showTasks(self, tasks: list) -> None:
        self.taskRows = []
        for (i, task) in enumerate(tasks):
            taskRow = TaskRow(self.viewPort, task, lambda t=task: self.unhighlightAndSelectTask(t))
            taskRow.grid(row=i, column=0, sticky= tk.W+tk.E)
            self.taskRows.append(taskRow)

        self.recordLabel.config(text=str(len(tasks)) + " tasks found")

    def unhighlightAndSelectTask(self, task) -> None:
        for tr in self.taskRows:
            if task is not None and tr.rowid == task["rowid"]:
                tr.highlight()
            else:
                tr.unhighlight()

        self.selectTask(task)

class TaskRow(tk.LabelFrame):
    def __init__(self, parentFrame: tk.Frame, task, select):
        super().__init__(parentFrame)
        self.select = select
        self.rowid = task["rowid"]

        self.taskName = tk.Label(self, text=task["Task"])
        self.taskName.grid(row=0, column=0, sticky = tk.W)

        self.category = tk.Label(self, text=task["Category"], font=("helvetica", 8))
        self.category.grid(row=1, column=0, sticky=tk.W)

        self.visible = [self, self.taskName, self.category]
        for o in self.visible:
            o.bind("<1>", lambda _: self.selectAndHighlight())

        self.unhighlight()

    def selectAndHighlight(self) -> None:
        self.select()
        self.highlight()

    def highlight(self) -> None:
        for w in self.visible:
            w.config(bg="lightblue")

    def unhighlight(self) -> None:
        for w in self.visible:
            w.config(bg="white")
