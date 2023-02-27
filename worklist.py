#!/usr/bin/python3

import sqlite3
import tkinter as tk
import tkinter.ttk
from tkinter import messagebox
import datetime
import time
import sys, os
import numpy as np
import re
from dateutil.relativedelta import relativedelta

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

###########################################

class WorklistWindow():
  def __init__(self, databasePath):
    self.os = sys.platform

    self.db = DatabaseManager(databasePath)

    #Tkinter stuff
    self.root = tk.Tk()

    self.setupWindow()

    # Start the program
    self.root.mainloop()

  ######################################################
  # GUI setup functions

  # Setup up the gui and load tasks
  def setupWindow(self):

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

    self.setupFrames()
    self.loadTasks()
    self.setupFilters()

    self.lb = TaskList(self.taskDisplayFrame, self.font, self.onSelect, lambda event: self.timeButton.invoke())

    #Setup the lower half of the window
    self.setupTimer()
    self.setupEntryBoxes()
    self.setupButtons()

    # These aren't all the keybindings, but they're all the ones the user should notice
    # Other keybindings mostly just make the app behave how you'd expect
    self.root.bind("<Control-q>", lambda event: self.root.destroy())
    self.root.bind("<Control-w>", lambda event: self.root.destroy())
    self.root.bind("<Control-n>", lambda event: self.newTaskButton.invoke())
    self.root.bind("<Control-f>", lambda event: self.searchBox.focus())

    self.refreshAll()

  def setupFrames(self):

    # Frame to hold the tasklist display and associated frames and widgets
    self.taskDisplayFrame = tk.Frame(self.root)
    self.taskDisplayFrame.grid(pady=self.padscale * 4, padx=self.padscale * 4)

    # Frame for the filters above the listbox displaying tasks
    self.filterFrame = tk.Frame(self.taskDisplayFrame)
    self.filterFrame.grid(sticky="W", pady=self.padscale * (6,2), padx=self.padscale * 2)

    # Frame for all the buttons and boxes below the tasklist pane
    self.interactiveFrame = tk.Frame(self.root)
    self.interactiveFrame.grid(pady=self.padscale * 4)

    # Frame for the calendar
    self.calendarFrame = Calendar(self.interactiveFrame, self.font, self.filterDate)
    self.calendarFrame.grid(row=0, column=3, pady=self.padscale * 4, padx=self.padscale * 4)

    # Timer and its button
    self.timerFrame = tk.Frame(self.interactiveFrame)
    self.timerFrame.grid(row=0, column=0, padx=self.padscale * [0,30])

    # Entry boxes and labels
    self.entryFrame = tk.Frame(self.interactiveFrame)
    self.entryFrame.grid(row=0, column=1)

    # For save button, etc. below entry boxes
    self.entryButtonFrame = tk.Frame(self.interactiveFrame)
    self.entryButtonFrame.grid(row=1, column=1)

    # The buttons to the right of the entry boxes, eg. backup
    self.adminButtonFrame = tk.Frame(self.interactiveFrame)
    self.adminButtonFrame.grid(row=0, column=2, padx=self.padscale * 4)

  def setupFilters(self):
    #Add filters at top
    self.filterBoxes = {}

    #Get categories
    self.catBox = tk.ttk.Combobox(self.filterFrame)
    self.catBox.bind("<KeyRelease>", lambda event: [self.completeBox(event, self.categories), self.refreshTasks()])
    self.filterBoxes["Category"] = [" LIKE ", self.catBox]

    #Finished/unfinished filter
    self.statusBox = tk.ttk.Combobox(self.filterFrame, values=["Any", "O", "X"], state="readonly")
    self.filterBoxes["O"] = [" == ", self.statusBox]

    #Task name search box
    self.searchBox = tk.Entry(self.filterFrame, width=25)
    self.searchBox.bind("<KeyRelease>", self.refreshTasks)
    self.filterBoxes["Task"] = [" LIKE ", self.searchBox]

    #Filter for available work vs. all
    self.availableBox = tk.ttk.Combobox(self.filterFrame,
                                        values=["Any Availability", "Available Now"],
                                        state="readonly")
    self.filterBoxes["NextAction"] = [" <= ", self.availableBox]

    self.refreshFilterCategories()
    self.refreshCategories()
    self.setDefaultFilters()

    #Bind so new selection refreshes, set width and pad, pack
    for (header, [operator, filterBox]) in list(self.filterBoxes.items()):
      # Task search box is already configured
      if header != "Task":
        filterBox.config(width=max([len(val) for val in filterBox["values"]]))
        filterBox.bind("<<ComboboxSelected>>", self.refreshTasks)
        filterBox.bind("<FocusOut>", self.clearComboHighlights)

      filterBox.pack(side=tk.LEFT, padx=self.padscale * 3)

    #Button to reset to default filters
    self.defaultFiltersButton = tk.Button(self.filterFrame,
                                          text="Reset Filters",
                                          command = self.resetFilters)
    self.defaultFiltersButton.pack(side=tk.LEFT, padx=self.padscale * 3)


  def setupTimer(self):
    #Timer and button to start/stop
    self.timeLabel = tk.Label(self.timerFrame, text="0:00:00", font=self.font)
    self.timeLabel.grid()

    self.timeButton = tk.Button(self.timerFrame, text="Start", command=self.toggleTimer)
    self.timeButton.grid()
    self.timeButton.bind("<Return>", self.toggleTimer)
    self.timing = False

  def setupEntryBoxes(self):
    self.editColumns = ["Category", "Task", "Time", "Used", "NextAction", "DueDate", "Flex", "Notes"]

    self.entryBoxes = {}
    self.entryLabels = {}

    #Add inputs below list
    for i, header in enumerate(self.editColumns):
      self.entryLabels[header] = tk.Label(self.entryFrame, text=header)
      self.entryLabels[header].grid(sticky="W",row=i, column=0)

      if header == "Category":
        self.entryBoxes[header] = tk.ttk.Combobox(self.entryFrame)
        self.entryBoxes[header].bind("<FocusOut>", self.clearComboHighlights)
        self.entryBoxes[header].bind("<KeyRelease>", lambda event: self.completeBox(event, self.categories))
        self.entryBoxes[header].bind("<Return>",
                                     lambda event: self.entryBoxes["Category"].icursor(tk.END))
      elif header == "Flex":
        self.entryBoxes[header] = tk.ttk.Combobox(self.entryFrame,
                                                  values=["Y","N"],
                                                  state="readonly")
        self.entryBoxes[header].bind("<FocusOut>", self.clearComboHighlights)
      elif header == "Notes":
        self.entryBoxes[header] = tk.Text(self.entryFrame, height=10, wrap="word")
        self.entryBoxes[header].bind("<Tab>", self.focusNextWidget)
      else:
        if header in ["DueDate", "NextAction"]:
          self.entryBoxes[header] = DateEntry(self.entryFrame, self.notify)
        else:
          self.entryBoxes[header] = tk.Entry(self.entryFrame)
        self.entryBoxes[header].bind("<Return>", self.save)

      self.entryBoxes[header].grid(sticky="NW",row=i, column=1, pady=self.padscale * 1)
      self.entryBoxes[header].config(width=60, font=self.font)

    self.checkDone = tk.StringVar()
    self.doneCheck = tk.Checkbutton(self.entryButtonFrame,
                                    text="Done",
                                    variable=self.checkDone,
                                    onvalue="X",
                                    offvalue="O")
    self.doneCheck.grid(row=0, column=0)
    self.doneCheck.deselect()

  def setupButtons(self):
    #Add buttons to interact
    self.saveButton = tk.Button(self.entryButtonFrame, text="Save Task", command=self.save)
    self.saveButton.grid(row=0, column=1)
    self.saveButton.bind("<Return>", self.save)

    self.newTaskButton = tk.Button(self.entryButtonFrame, text="New Task", command = self.newTask)
    self.newTaskButton.grid(row=0, column=2)
    self.newTaskButton.bind("<Return>", self.newTask)

    self.deleteButton = tk.Button(self.entryButtonFrame,
                                  text="Delete Task",
                                  command = self.deleteSelected)
    self.deleteButton.grid(row=0, column=3)
    self.deleteButton.bind("<Return>", self.deleteSelected)

    self.messageLabel = tk.Label(self.interactiveFrame, text="")
    self.messageLabel.grid(column=0, columnspan=3)

    # todo doesn't really belong here. The multiedit stuff should probably move down below with the other task buttons too
    self.duplicateButton = tk.Button(self.adminButtonFrame,
                                     text="Duplicate Task",
                                     command=self.duplicateTask)
    self.duplicateButton.grid(sticky="W")

    self.multiEdit = tk.IntVar()
    self.multiEditButton = tk.Checkbutton(self.adminButtonFrame,
                                          text="Edit multiple",
                                          command = self.multiEditConfig,
                                          variable=self.multiEdit,
                                          onvalue=True,
                                          offvalue=False)
    self.multiEditButton.grid(sticky="W")

    # todo because of where these two are, it's inconvenient to tab through when using the keyboard instead of a mouse
    self.intervalBox = tk.ttk.Combobox(self.adminButtonFrame,
                                       values=["", "Weekly", "Biweekly", "Monthly", "Annually"],
                                       state="disabled")
    self.intervalBox.grid(sticky="W")
    self.intervalBox.bind("<FocusOut>", self.clearComboHighlights)

    self.repetitionBox = tk.Entry(self.adminButtonFrame, width=2, state="disabled")
    self.repetitionBox.grid(sticky="W")

  ######################################################
  # GUI update functions

  #Clears the annoying highlighting from all comboboxes
  def clearComboHighlights(self, event=tk.Event):
    for header in ["Category","O","NextAction"]:
      self.filterBoxes[header][1].selection_clear()

    for header in ["Category", "Flex"]:
      self.entryBoxes[header].selection_clear()

    self.intervalBox.selection_clear()

  def resetFilters(self, eventy=tk.Event):
    self.refreshAll()
    self.setDefaultFilters()
    self.refreshTasks()
    if self.multiEdit.get():
      self.multiEdit.set(False)
      self.multiEditConfig()

  #Sets filters to default
  def setDefaultFilters(self, event=tk.Event):
    self.catBox.current(0)
    self.statusBox.current(1)
    self.availableBox.current(1)
    self.overwriteEntryBox(self.searchBox, "")

  # todo there should be a manual filter box to filter by start/end date so that this is persistent and can be layered with other filters.
  # Filter to only show tasks available on the passed date
  def filterDate(self, date):
    self.lb.selection = None
    self.resetFilters()
    criteria = ["O == 'O'", "NextAction <= '{}'".format(date2YMDstr(date)), "DueDate >= '{}'".format(date2YMDstr(date))]
    self.loadTasks(criteria)
    self.lb.showTasks(self.loadedTasks)

  #Set up filters to match the selected task (ideally catching all of a repeating task)
  def multiEditConfig(self):
    if self.multiEdit.get():
      #Enter special mode
      self.timeButton.config(state="disabled")
      self.entryBoxes["Used"].config(state="readonly")

      if self.lb.selection is not None:
        #Enter multiedit mode
        self.previousSearch = self.searchBox.get()
        self.overwriteEntryBox(self.searchBox, self.loadedTasks[self.lb.selection]["Task"])
        self.catBox.set(self.loadedTasks[self.lb.selection]["Category"])
        self.statusBox.current(1)
        self.availableBox.current(0)

        for widget in [self.catBox, self.statusBox, self.availableBox]:
          widget.config(state="disabled")

        self.refreshTasks()
        self.notify("Multiedit mode")
      else:
        #If no task selected, go to repeating task mode
        self.intervalBox.config(state="readonly")
        self.repetitionBox.config(state="normal")
        #Disable all widgets in the list
        for widget in [item[1][1] for item in list(self.filterBoxes.items())] + [self.lb, self.defaultFiltersButton]:
          widget.config(state="disabled")

        self.notify("Repeating mode")
    else:
      #Return to normal mode
      self.timeButton.config(state="normal")
      self.entryBoxes["Used"].config(state="normal")

      if self.lb.selection is not None:
        #Leave multiedit mode
        self.overwriteEntryBox(self.searchBox, self.previousSearch)
        self.setDefaultFilters()

        for widget in [self.statusBox, self.availableBox]:
          widget.config(state="readonly")

        self.catBox.config(state="normal")

        self.refreshTasks()
      else:
        #Leave repeating mode
        self.intervalBox.set("")
        self.overwriteEntryBox(self.repetitionBox,"")

        for widget in [self.intervalBox, self.repetitionBox]:
          widget.config(state="disabled")

        for widget in [self.lb, self.defaultFiltersButton, self.searchBox]:
          widget.config(state="normal")

        for widget in [item[1][1] for item in list(self.filterBoxes.items())]:
          if widget not in [self.searchBox, self.catBox]:
            widget.config(state="readonly")

        self.catBox.config(state="normal")

      self.notify("Normal mode")

  # todo timer should probably be its own class - turns out this is tougher than you'd expect because of the links between the time, entryboxes and tasklist
  def toggleTimer(self, event=tk.Event):
    if not self.timing:
      self.timeButton.config(text="Stop")
      self.timing = True
      self.startTime = time.strftime("%H:%M:%S")
      self.runTimer()
    else:
      self.timeButton.config(text="Start")
      self.timing = False
      try:
        self.overwriteEntryBox(self.entryBoxes["Used"], round(self.timerVal.total_seconds()/60))
        self.save()
      except AttributeError:
        #Empty task
        pass

  def runTimer(self):
    timeFormat = "%H:%M:%S"
    if self.timing:
      # TODO would be better to handle this by accounting for date rather than just fudging the days
      runTime = (datetime.datetime.strptime(time.strftime(timeFormat), timeFormat)
                 - datetime.datetime.strptime(self.startTime, timeFormat))
      # If the timer is run through midnight it goes negative. This fixes it.
      if runTime.days < 0:
        runTime = runTime + datetime.timedelta(days=1)

      try:
        if self.lb.selection is None:
          raise ValueError("Cannot time an empty task")
        self.timerVal = (runTime
                         + datetime.timedelta(minutes=(self.loadedTasks[self.lb.selection]["Used"] or 0)))
        self.timeLabel.config(text=str(self.timerVal))
        self.root.after(1000, self.runTimer)
      except ValueError as e:
        self.notify(e)
        self.timerVal = None
        self.timeButton.invoke()

  def completeBox(self, event, sourceList):
    #Don't run when deleting, or when shift is released
    if event.keysym not in ["BackSpace", "Shift_L", "Shift_R"]:
      box = event.widget
      cursorPos = box.index(tk.INSERT)
      current = box.get()[0:cursorPos]
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
            #Iterating out the end of one of the options
            pass

          i-=1

          #only if found text longer than current in common
          if i > len(current) - 1:
            box.insert(tk.END, options[0][cursorPos:i+1])

          box.select_range(cursorPos, tk.END)
          box.icursor(tk.END)

  #Updates the categories in the category filter
  def refreshFilterCategories(self):
    self.filterCategories = list(set([task["Category"] for task in self.loadedTasks]))
    self.filterCategories.sort()
    self.filterCategories = ["All categories"] + self.filterCategories
    self.catBox.config(values=self.filterCategories)

  # Updates the suggested categories in the category entrybox
  def refreshCategories(self):
    self.categories = self.db.getCategories()
    try:
      self.entryBoxes["Category"].config(values=self.categories)
    except AttributeError:
      #Fails on setup
      pass

  def getSearchCriteria(self):

    criteria = []

    #Iterate over all stored filter boxes, adding their criteria
    for (header, [operator, filterBox]) in list(self.filterBoxes.items()):

      criterion = None
      quote     = None

      #If Task searchbox and has string, or if on non-null combobox option
      if header in ["Category", "Task"]:
        #These ifs have to be nested without and because we don't want header == "Task" but not filterBox.get() to fall through
        if filterBox.get():
          # Don't even bother when "Category" is "All"
          if not (header == "Category" and not filterBox.current()):
            # On multiedit, use exact matches only (unless you manually enter the % wildcard)
            if self.multiEdit.get():
              criterion = filterBox.get()
              quote     = "'"
            else:
              criterion = filterBox.get()
              quote     = "'%"
      elif filterBox.current():
        if header == "NextAction":
          criterion = todayStr()
          quote     = "'"
        else:
          criterion = filterBox.get()
          quote     = "'"

      if criterion is not None:
        criteria.append(header + operator + surround(escapeSingleQuotes(criterion), quote))

    return criteria

  def refreshTasks(self, event=tk.Event):
    #Remember which task was selected
    if self.lb.selection not in [None, -1]:
      self.selected_rowid = self.loadedTasks[self.lb.selection]["rowid"]

    criteria = self.getSearchCriteria()
    self.loadTasks(criteria)
    self.lb.showTasks(self.loadedTasks)

    if self.lb.selection not in [None, -1]:
      #Find the previously selected task
      previousSelection = None
      for i, task in enumerate(self.loadedTasks):
        if task["rowid"] == self.selected_rowid:
          previousSelection = i
          break

      self.lb.selection = previousSelection

      if previousSelection is not None:
        self.lb.selectListboxItem(self.lb.selection)
      else:
        self.clearEntryBoxes()

    #Update the category filterbox to only show available categories
    self.refreshFilterCategories()

  def confirmDiscardChanges(self):
    if self.nonTrivialChanges():
      selection = tk.messagebox.askyesnocancel(title="Save before switching?",
                                               message = "Do you want to save your changes to '{}' before switching?".format(self.loadedTasks[self.lb.selection]["Task"]))
      if selection is True:
        self.save()
      elif selection is None:
        self.resetListboxSelection()
        return False
      else:
        self.notify("Discarded changes")

    return True

  def confirmCancelTimer(self):
    try:
      if self.timing:
        self.entryBoxes["Category"].focus()
        selection = tk.messagebox.askyesnocancel(title="Save before switching?",
                                                 message="Do you want to save the timer for '{}' before switching?".format(self.loadedTasks[self.lb.selection]["Task"]))
        if selection is True:
          pass
        elif selection is None:
          self.resetListboxSelection()
          return False
        else:
          self.timerVal = None
          self.notify("Discarded timer")

        self.timeButton.invoke()
      return True
    except AttributeError:
      #On startup the timer isn't setup yet
      pass

  def newTask(self, event=tk.Event):
    #This needs to be here to prevent the message box from dumping us into self.lb on exit and calling onSelect()
    self.entryBoxes["Category"].focus()
    self.clearEntryBoxes()
    self.notify("Creating new entry")
    self.lb.selection_clear(0,'end')
    self.lb.selection = None

  #Bound to the Tab key for Text box, so that it will cycle widgets instead of inserting a tab character
  def focusNextWidget(self, event):
    event.widget.tk_focusNext().focus()
    return("break")

  def clearEntryBoxes(self):
    if self.confirmDiscardChanges():
      try:
        #Nothing selected, just clear the box
        self.checkDone.set("O")
        self.entryBoxes["Flex"].set("")
        self.timeLabel.config(text="0:00:00")
        for header in self.editColumns:
          self.overwriteEntryBox(self.entryBoxes[header], "")
      except AttributeError:
        #fails on startup
        pass
    else:
      raise PermissionError("Cancelled by user")

  # todo either this or multiEditConfig is sometimes leaving "Used" in readonly
  def overwriteEntryBox(self, entry, text):
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
      entry.config(state="readonly")

  #Updates the entry boxes when a task is selected
  def onSelect(self, event=tk.Event):
    # todo would be better to keep track of the actual rowid rather than just the selection index
    self.messageLabel.config(text="")
    try:
      if self.lb.selection != self.lb.curselection()[0]:
        if self.confirmDiscardChanges():
          if self.confirmCancelTimer():
            self.lb.selection = self.lb.curselection()[0]
          else:
            # TODO wtf lol
            raise PermissionError

          #todo this could be a function "update entryBoxes" or something
          for (header, entry) in [(header, self.entryBoxes[header]) for header in self.editColumns]:
            if header == "Flex":
              entry.set(self.loadedTasks[self.lb.selection][header])
            else:
              self.overwriteEntryBox(entry, self.loadedTasks[self.lb.selection][header])

          self.checkDone.set(self.loadedTasks[self.lb.selection]["O"])
      if not self.timing:
        self.timeLabel.config(text=str(datetime.timedelta(minutes=(self.loadedTasks[self.lb.selection]["Used"] or 0))))

    except IndexError:
      #This happens when you select into the entry boxes
      pass
    except PermissionError:
      pass

  # todo the refresh message gets clobbered here somewhere
  def refreshAll(self, event=tk.Event):
    self.refreshCategories()
    self.refreshFilterCategories()
    self.updateLoadsToday()
    self.calendarFrame.updateCalendar(self.db.getTasks4Workload())
    self.refreshTasks()

  def notify(self, msg):
    try:
      self.messageLabel.config(text=msg)
    except AttributeError as e:
      # Fails on startup
      pass
    print(msg)

  ######################################################
  # Calculation functions

  def getSelectedInterval(self):
    selection = self.intervalBox.get()
    if selection == "":
      raise ValueError("No interval selected")
    elif selection == "Weekly":
      interval = relativedelta(weeks=1)
    elif selection == "Biweekly":
      interval = relativedelta(weeks=2)
    elif selection == "Monthly":
      interval = relativedelta(months=1)
    elif selection == "Annually":
      interval = relativedelta(years=1)

    return interval

  #Gets the text in the passed entryBox
  def getEntry(self, entryBox):
    try:
      return entryBox.get()
    except TypeError:
      #Notes
      return entryBox.get('1.0','end')[:-1]

  #validate data
  def validateRow(self, row):
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
  def calculateRow(self, inRow, event=tk.Event):
    today = todayStr()

    newRowDict = {}
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
  def loadTasks(self, criteria=[]):
    self.loadedTasks = self.db.getTasks(criteria)
    if len(self.loadedTasks) == 0:
      self.notify("No tasks found")

  #Deletes the task selected in the listbox from the database
  def deleteSelected(self, event=tk.Event):
    taskToDelete = self.loadedTasks[self.lb.selection]
    try:
      deleted = False
      if not self.multiEdit.get():
        if(tk.messagebox.askyesno(title="Confirm deletion",
                                  message="Are you sure you want to delete '{}'?".format(
                                            taskToDelete["Task"]))):
          self.db.deleteByRowid(taskToDelete["rowid"])
          self.notify("Deleted '{}'".format(taskToDelete["Task"]))
          deleted = True
      else:
        #Delete multi
        if(tk.messagebox.askyesno(title="Confirm deletion",
                                  message="This will DELETE ALL TASKS matching: '{}' in '{}'.\
                                           Are you sure you want to proceed?".format(
                                            taskToDelete["Task"],
                                            taskToDelete["Category"]))):
          self.db.deleteByNameCat(taskToDelete["Task"], taskToDelete["Category"])
          self.notify("Deleted '{}'".format(taskToDelete["Task"]))
          deleted = True

      # Only need to do this if deleted a task
      if deleted:
        self.db.commit()

        self.clearEntryBoxes()
        self.refreshAll()

        self.newTask()
        self.refreshTasks()

        #Prevents listbox from grabbing focus and selecting first task
        self.lb.selection = -1

    except TypeError:
      self.notify("Cannot delete - none selected")

  #Save the current state of the entry boxes for that task
  def save(self, event=tk.Event()):
    if self.confirmCancelTimer():
      try:
        if self.lb.selection is None:
          self.createTaskFromInputs()
        else:
          self.updateSelectedTask()

        #Refresh the screen
        self.refreshAll()

        self.notify("Task saved")
      except ValueError as e:
        #Something wrong with the inputs given
        self.notify(e)
      except PermissionError as e:
        #updateSelectedTask() cancelled by user
        self.notify(e)

  # todo a more elegant way of handling repeating tasks than just creating a bunch of duplicates. Maybe a task that duplicates itself a number of days in the future when completed?
  def createTaskFromInputs(self):
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

    if self.multiEdit.get():
      #Get interval between tasks and number of tasks to create
      interval = self.getSelectedInterval()
      try:
        repetitions = int(self.repetitionBox.get())
        if not repetitions > 0:
          raise ValueError
      except:
        raise ValueError("Invalid repetition count")
    else:
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
    # todo would be more elegant to just select the newly created task. This is tricky because the newly created task isn't necessarily shown by the current filters (e.g. if it is far in the future)
    self.clearEntryBoxes()

    if self.multiEdit.get():
      #If in multiedit mode, set back to normal mode
      self.multiEdit.set(False)
      self.multiEditConfig()

  # Like updateSelectedTask, but you pass the updated task in rather than pulling from input
  # Doesn't commit the changes, so you can loop without a huge overhead
  def updatePassedTask(self, row):
    #Find which columns were changed and how
    changes = []
    newRow = {}

    self.validateRow(row)
    newRow = self.calculateRow(row)
    for header in self.db.headers:
      changes.append(" {} = '{}' ".format(header, escapeSingleQuotes(str(newRow[header]))))

    criteria = ["rowid = {}".format(row["rowid"])]

    self.db.updateTasks(criteria, changes)

  def nonTrivialChanges(self):
    changes = self.getChanges()
    if len(changes) == 1 and "DaysLeft" in changes[0]:
      return False
    elif len(changes) == 2 and "DaysLeft" in changes[0] and "TotalLoad" in changes[1]:
      return False
    elif len(changes) == 0:
      return False
    else:
      return True

  def getChanges(self):
    changes = []
    if self.lb.selection in [None, -1]:
      pass
    else:
      #Find which columns were changed and how

      newRow = {}
      oldRow = dict(self.loadedTasks[self.lb.selection])

      for (header, old) in [(header, oldRow[header]) for header in self.db.headers]:
        if header in self.editColumns + ["O"]:
          #This is a checkbox and not in the edit list
          if header == "O":
            # double ifs so "O" can't fall through
            if self.multiEdit.get():
              new = old
            else:
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
            if not (self.multiEdit.get() and header == "Load"):
              changes.append(" {} = '{}' ".format(header, escapeSingleQuotes(str(new))))

    return changes

  # todo would be nice if multiupdate could change names with a delta, not just dates UPDATE tablename SET var = REPLACE(string_to_modify, find_this, replace_with_this) WHERE searchCriteria
  # Update the currently selected task with values from the entry boxes
  # If multiEdit is enabled, updates all tasks matching current filter criteria
  # If a task is passed in ("row"), as a dict or sqlite3.Row, updates this instead, by rowid
  def updateSelectedTask(self):
    # verify before multiupdating
    if self.multiEdit.get() and not tk.messagebox.askyesno(title="Confirm multiupdate",
                                                           message="Are you sure you want to change all tasks matching: '{}'?".format(self.searchBox.get())):
      #User cancelled
      raise PermissionError("Cancelled task update")

    changes = self.getChanges()

    if changes:
      if self.multiEdit.get():
        criteria = self.getSearchCriteria()
      else:
        criteria = ["rowid = {}".format(self.loadedTasks[self.lb.selection]["rowid"])]

        # todo messy
        # Dump the time worked to external time tracker
        for change in changes:
            if change.find("Used") != -1:
                timediff = int(re.findall(r"(\d+)", change)[0])
                with open("timesheet.csv", "a") as f:
                    f.write("{}, {}, {}, {}\n".format(todayStr(), self.loadedTasks[self.lb.selection]["Category"], timediff, self.loadedTasks[self.lb.selection]["Task"]))

      self.db.updateTasks(criteria, changes)

      self.db.commit()

      if self.multiEdit.get():
        #So the new search still includes the task, even if you change the name or category
        self.overwriteEntryBox(self.searchBox, self.getEntry(self.entryBoxes["Task"]))
        self.catBox.set(self.getEntry(self.entryBoxes["Category"]))

  def duplicateTask(self):

    oldSelection = self.lb.selection
    self.lb.selection = None
    self.save()
    self.lb.selectListboxItem(oldSelection)
    self.notify("Duplicated task")

  #scans all tasks and updates using calculateRow()
  def updateLoadsToday(self, event=tk.Event):
    try:
      #backup task list
      oldTasks = self.loadedTasks
    except AttributeError:
      pass

    self.loadTasks(["O == 'O'","NextAction <= '{}'".format(todayStr())])

    #Don't commit until the end - saves a few seconds each time
    for task in self.loadedTasks:
      self.updatePassedTask(task)
    self.db.commit()

    try:
      #put original task list back
      self.loadedTasks = oldTasks
    except UnboundLocalError:
      pass

    try:
      self.notify("Workloads refreshed")
    except AttributeError:
      #This fails during startup, which is good because we don't want the message anyways
      pass

class DatabaseManager():
  def __init__(self, databasePath):
    self.conn = sqlite3.connect(databasePath)
    self.conn.row_factory = sqlite3.Row

    self.c = self.conn.cursor()
    self.cwrite = self.conn.cursor()

  def commit(self):
    self.conn.commit()

  #Loads the tasks by searching the database with the criteria specified
  def getTasks(self, criteria=[]):
    #Super basic SQL injection check
    if True in [';' in s for s in criteria]:
      raise ValueError("; in SQL input!")

    command = "SELECT *, rowid FROM worklist"

    if criteria:
      command += " WHERE "
      command += " AND ".join(criteria)

    command += " ORDER BY DueDate;"

    self.c.execute(command)

    tasks = self.c.fetchall()

    # todo only needs to be done on startup
    self.headers = [description[0] for description in self.c.description]

    return tasks

  def getTasks4Workload(self):
    self.cwrite.execute("SELECT NextAction, DueDate, Left FROM worklist WHERE O == 'O' ORDER BY DueDate;")
    return self.cwrite.fetchall()

  #Updates the categories in the category filter
  def getCategories(self):
    self.cwrite.execute("SELECT DISTINCT Category FROM worklist ORDER BY Category;")
    return [line["Category"] for line in self.cwrite.fetchall()]

  def deleteByRowid(self, rowid):
    self.cwrite.execute("DELETE FROM worklist WHERE rowid == ?", [rowid])

  def deleteByNameCat(self, taskName, category):
    self.cwrite.execute("DELETE FROM worklist WHERE Task==? AND Category==? AND O='O'", [taskName, category])

  def checkSqlInput(self, sqlString):
    if type(sqlString) not in [int, float, type(None)]:
      #todo a better way of cleaning input
      badChars = [';']
      if any((c in badChars) for c in sqlString):
        raise ValueError("Bad SQL input: {}".format(sqlString))

  def updateTasks(self, criteria, changes):
    for string in criteria + changes:
      self.checkSqlInput(string)

    command = "UPDATE worklist SET "
    command += ", ".join(changes)
    command += " WHERE "
    command += " AND ".join(criteria)
    command += ";"

    self.cwrite.execute(command)

  def createTask(self, headers, vals):
    for string in headers + vals:
      self.checkSqlInput(string)

    cleanVals = []
    # Cleans quotes in SQL input
    for val in vals:
      try:
        cleanVals.append(surround(escapeSingleQuotes(str(val)), "'"))
      except TypeError:
        cleanVals.append(str(val))

    command = "INSERT INTO worklist ("

    command += ", ".join(headers)
    command +=  " ) VALUES ("

    command += ", ".join(cleanVals)
    command += " );"

    self.cwrite.execute(command)

# todo put the next action / due date at a specific time?
# todo add buttons to scroll the calendar forward week-by-week
# todo Days of the week shown should be user-configurable (M-F vs. student schedule lol, or freelance).
# eg. thisDay["LoadLabel"].bind("<Button-1>", CALLBACK)
# Set up the calendar display to show estimated workload each day for a several week forecast
class Calendar(tk.Frame):
  def __init__(self, parentFrame, parentFont, dateCallback):
    tk.Frame.__init__(self, parentFrame)

    self.numweeks = 4
    self.dateCallback = dateCallback

    #Build the calendar out of labels
    self.calendar = []

    #Add day of week names at top, but won't change so don't save
    for i, day in enumerate(["Mon", "Tue", "Wed", "Thu", "Fri"]):
      tk.Label(self, font=parentFont + ("bold",), text=day).grid(row=0, column=i, padx=4, pady=4)

    for week in range(self.numweeks):
      thisWeek = []
      for day in range(5):
        thisDay = {}
        # todo *Sometimes* this significantly slows boot time. Could maybe cut down on labels by having dates all in a row for each week, but lining up with loads could be tricky. First row changes colour, so could do each date row below the first as a multi-column label.
        #Alternate date labels and workloads
        thisDay["DateLabel"] = tk.Label(self, font=parentFont)
        thisDay["DateLabel"].grid(row=2*week + 1, column=day, padx=4, pady=4)
        thisDay["LoadLabel"] = tk.Label(self, font=parentFont)
        thisDay["LoadLabel"].grid(row=2*week + 2, column=day, padx=4, pady=4)
        thisWeek.append(thisDay)
      self.calendar.append(thisWeek)

  def updateCalendar(self, openTasks):

    self.calculateDayLoads(openTasks)

    today = todayDate()
    thisMonday = today - datetime.timedelta(days=today.weekday())
    hoursLeftToday = max(0, min(7, 16 - (datetime.datetime.now().hour + datetime.datetime.now().minute/60)))
    for week in range(self.numweeks):
      for day in range(5):
        thisDay = self.calendar[week][day]
        thisDate = thisMonday + datetime.timedelta(days=day, weeks=week)
        thisDay["Date"] = thisDate
        thisDay["DateLabel"].bind("<Button-1>", lambda event, a = thisDay["Date"]: self.dateCallback(a))
        thisDay["LoadLabel"].bind("<Button-1>", lambda event, a = thisDay["Date"]: self.dateCallback(a))
        thisDay["DateLabel"].config(text=thisDate.strftime("%b %d"))
        if thisDate == today:
          thisDay["DateLabel"].config(bg="lime")
        else:
          thisDay["DateLabel"].config(bg="#d9d9d9")
        if thisDate >= today:
          hoursThisDay = self.getDayTotalLoad(date2YMDstr(thisDate)) / 60
          thisDay["LoadLabel"].config(text=str(round(hoursThisDay,1)),
                                      bg=greenRedScale(0,(7 if thisDate != today else max(0.1, hoursLeftToday)),hoursThisDay))
        else:
          thisDay["LoadLabel"].config(text="", bg="#d9d9d9")

  def calculateDayLoads(self, openTasks):
    # Get a list of all unfinished tasks with start dates no more than self.numweeks in the future, sorted from soonest due date to latest
    today = todayDate()
    thisFriday = today - datetime.timedelta(days=today.weekday() + 4)
    lastRenderedDate = thisFriday + datetime.timedelta(weeks=self.numweeks-1)
    endDate = date2YMDstr(lastRenderedDate)
    relevantTasks = [task for task in openTasks if task["NextAction"] <= endDate]

    # Iterate over the list of tasks (starting from soonest due date), distributing time evenly (each day gets time remaining / # days remaining) over days from max(today, start date) to due date. If adding time would push day over 8 hours, only add up to 8 hours, and withold extra time within the task.
    self.dayLoads = {}
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
  def getDayTotalLoad(self, date):
    # Will raise an error if date is poorly formatted
    YMDstr2date(date)

    try:
      return self.dayLoads[date]
    except KeyError:
      return 0;

class DateEntry(tk.Entry):
  def __init__(self, parentFrame, notificationFunc):
    tk.Entry.__init__(self, parentFrame)
    self.notify = notificationFunc
    self.bind("<Tab>", self.convertDate)

  def convertDate(self, event=tk.Event):
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

class TaskList(tk.Listbox):
  def __init__(self, parentFrame, parentFont, onSelect, onDoubleClick):
    tk.Listbox.__init__(self, parentFrame,
                        selectmode=tk.SINGLE,
                        width = 140,
                        height=10,
                        font=parentFont,
                        selectbackground="SteelBlue1")

    #Get these columns from the database
    self.displayColumns = ["Category", "O", "Task", "Time", "Used", "Left", "NextAction", "DueDate", "Flex", "Load"]

    #Headers for the Listbox
    self.headerLabel = tk.Label(parentFrame, text="", font=parentFont)
    self.headerLabel.grid(sticky="W")
    self.grid(sticky="W")

    #The Listbox that allows us to view & select different tasks

    self.onSelect = onSelect
    self.selection = None

    #Lets you scroll with arrow keys
    self.bind("<Down>", self.onDown)
    self.bind("<j>", self.onDown)
    self.bind("<Up>", self.onUp)
    self.bind("<k>", self.onUp)
    self.bind("<<ListboxSelect>>", self.onSelect)
    self.bind("<FocusIn>", self.selectFirst)
    self.bind("<Double-1>", onDoubleClick)
    self.bind("<Return>", onDoubleClick)

    self.recordLabel = tk.Label(parentFrame, text="")
    self.recordLabel.grid(sticky="E")

  #Updates the listbox with the loaded tasks
  def showTasks(self, loadedTasks):
    #Max length of any item in each column, including headers
    self.maxlens = [max([len(str(row[column])) for row in loadedTasks] + [len(column)]) for column in self.displayColumns]
    #Forces these two to 50 to try and maintain some order
    self.maxlens[self.displayColumns.index("Task")] = 60

    #Adjust the listbox to be wide enough
    self.config(width=sum(self.maxlens) + len(self.maxlens) * 3)

    self.recordLabel.config(text=str(len(loadedTasks)) + " tasks found")

    #Create the header text
    headerline = ""
    for header, length in zip(self.displayColumns, self.maxlens):
      headerline += header.center(length) + " | "
    headerline = headerline[:-2]
    self.headerLabel.config(text = headerline)

    #delete tasks and reinsert
    self.delete(0,'end')
    for task in loadedTasks:
      line = ""
      for (header,length) in zip(self.displayColumns, self.maxlens):
        try:
          line += YMDstr2date(str(task[header])).strftime("%b %d, %y").ljust(length) + " | "
        except ValueError:
          line += ljusttrunc(str(task[header]), length) + " | "
      line = line[:-2]

      self.insert(tk.END,line)
      #Colour-coding!
      try:
        self.itemconfig(tk.END, {'bg': greenRedScale(0, 60, task["Load"])})
      except TypeError:
        #Fails for items where Load is None, eg. completed, not yet active
        pass

  def resetListboxSelection(self):
    self.selection_clear(0,'end')
    self.select_set(self.selection)

  #Lets you scroll with arrow keys
  def onDown(self, event):
    if self.selection < self.size() - 1:
      self.select_clear(self.selection)
      self.selectListboxItem(self.selection + 1)

  #Lets you scroll with arrow keys
  def onUp(self, event):
    if self.selection > 0:
      self.select_clear(self.selection)
      self.selectListboxItem(self.selection - 1)

  #When the listbox gains focus, select the first item
  def selectFirst(self, event=tk.Event):
    if self.selection is None:
      self.selectListboxItem(0)

  #Pretty self explanatory. Used because select_set doesn't trigger <<ListboxSelect>> event
  def selectListboxItem(self, item):
    self.select_set(item)
    self.onSelect()

def main():
  if len(sys.argv) > 1:
    worklist = WorklistWindow(sys.argv[1])
  elif os.path.isfile("worklist.db"):
    #default
    worklist = WorklistWindow("worklist.db")
  else:
    print("No worklist found and none specified.\nCreating new worklist.db")
    conn = sqlite3.connect("worklist.db")
    cur  = conn.cursor()
    # todo a better name for "Load" would be "CurrentLoad"
    cur.execute("CREATE TABLE worklist('Category' TEXT,'O' TEXT,'Task' TEXT,'Budget' INTEGER,'Time' INTEGER,'Used' INTEGER,'Left' INTEGER,'StartDate' TEXT,'NextAction' TEXT,'DueDate' TEXT,'Flex' TEXT,'DaysLeft' INTEGER,'TotalLoad' REAL,'Load' REAL,'Notes' TEXT,'DateAdded' TEXT)")
    cur.close()
    worklist = WorklistWindow("worklist.db")

#Like .ljust, but truncates to length if necessary
def ljusttrunc(text, length):
  return text[:length].ljust(length)

def greenRedScale(low, high, val):
  #linear interpolation bounded on [0,1]
  frac = max(0, min(1, (val - low) / (high - low)))
  if frac > 0.5:
    red = 255
    green = int((2-2*frac) * 255)
  else:
    red = int((2*frac) * 255)
    green = 255

  return "#{}{}00".format(str(hex(red)[2:]).rjust(2,'0'), str(hex(green)[2:]).rjust(2,'0'))

#Surrounds the string inner with string outer, reversed the second time, and returns the result
def surround(inner, outer):
  return outer + inner + outer[::-1]

#Double up single quotes in a string
def escapeSingleQuotes(text):
  return "".join([c if c != "'" else c+c for c in text])

# Takes a string "YYYY-MM-DD"
def daysBetween(d1, d2):
  d1 = YMDstr2date(d1)
  d2 = YMDstr2date(d2)
  return (d2 - d1).days

# takes strings "%Y-%m-%d"
# inclusive of start and end date
def workDaysBetween(d1, d2):
  return int(np.busday_count(d1, (YMDstr2date(d2) + datetime.timedelta(days=1))))

def YMDstr2date(dateString):
  return datetime.datetime.strptime(dateString, "%Y-%m-%d").date()

def date2YMDstr(dateVar):
  return dateVar.strftime("%Y-%m-%d")

def todayStr():
  return date2YMDstr(todayDate())

def todayDate():
  return datetime.date.today()

if __name__ == '__main__':
  main()
