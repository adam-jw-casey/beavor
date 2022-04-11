#!/usr/bin/python3

import sqlite3
import tkinter as tk
import tkinter.ttk
import tkinter.font as tkFont
from tkinter import messagebox
import datetime
import time
import subprocess
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
# todo integration to put tasks into google calendar would be cool or just have a way of marking a task as scheduled
# todo user-customizable settings (like font size, calendar colourscale)
# todo Dark mode toggle

###########################################

class TaskListWindow():
  def __init__(self, databasePath):
    self.os = sys.platform

    self.db = DatabaseManager(databasePath)

    #Tkinter stuff
    self.root = tk.Tk()

    self.setupWindow()

  # Start the program
  def runLoop(self):
    self.root.mainloop()

  ######################################################
  # GUI setup functions

  # Setup up the gui and load tasks
  def setupWindow(self):

    if self.os == "linux":
      self.root.attributes('-zoomed', True)
    else:
      #win32
      self.root.state("zoomed")

    self.root.winfo_toplevel().title("WORKLIST Beta")

    #Scale all padding by this multiplier (not tested lol)
    self.padscale = 1

    self.setupFrames()
    self.loadTasks()
    self.setupFilters()

    self.setupTaskListBox()

    #recalculate timediffs and loads
    self.updateLoadsToday()
    self.refreshTasks()

    #Setup the lower half of the window
    self.setupTimer()
    self.setupEntryBoxes()
    self.setupButtons()
    self.setupCalendar()

    self.setupKeybindings()

    self.refreshCategories()
    self.refreshFilterCategories()

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
    self.calendarFrame = tk.Frame(self.interactiveFrame)
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

  def setupTaskListBox(self):
    #Get these columns from the database
    self.displayColumns = ["Category", "O", "Task", "Time", "Used", "Left", "NextAction", "DueDate", "Flex", "Load", "Notes"]
    self.editColumns    = ["Category", "Task", "Time", "Used", "NextAction", "DueDate", "Flex", "Notes"]

    if self.os == "linux":
      self.font = ("Liberation Mono", 9)
    else:
      #win32
      self.font = ("Courier", 9)

    #Headers for the ListBox
    self.headerLabel = tk.Label(self.taskDisplayFrame, text="", font=self.font)
    self.headerLabel.grid(sticky="W")

    #The ListBox that allows us to view & select different tasks
    self.lb = tk.Listbox(self.taskDisplayFrame,
                         selectmode=tk.SINGLE,
                         width = 140,
                         height=18,
                         font=self.font,
                         selectbackground="SteelBlue1")
    self.lb.grid(sticky="W")

    # todo <Return> on a tasklistbox item should start timer
    #Lets you scroll with arrow keys
    self.lb.bind("<Down>", self.onDown)
    self.lb.bind("<j>", self.onDown)
    self.lb.bind("<Up>", self.onUp)
    self.lb.bind("<k>", self.onUp)
    self.lb.bind("<<ListboxSelect>>", self.onSelect)
    self.lb.bind("<FocusIn>", self.selectFirst)
    self.lb.bind("<Double-1>", lambda event: self.timeButton.invoke())

    self.selection = None

    self.recordLabel = tk.Label(self.taskDisplayFrame, text="")
    self.recordLabel.grid(sticky="E")

  def setupTimer(self):
    #Timer and button to start/stop
    self.timeLabel = tk.Label(self.timerFrame, text="0:00:00", font=self.font)
    self.timeLabel.grid()

    self.timeButton = tk.Button(self.timerFrame, text="Start", command=self.toggleTimer)
    self.timeButton.grid()
    self.timeButton.bind("<Return>", self.toggleTimer)
    self.timing = False

  def setupEntryBoxes(self):
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
        self.entryBoxes[header] = tk.Text(self.entryFrame, height=6, wrap="word")
        self.entryBoxes[header].bind("<Tab>", self.focusNextWidget)
      else:
        self.entryBoxes[header] = tk.Entry(self.entryFrame)
        self.entryBoxes[header].bind("<Return>", self.save)

      if header in ["DueDate", "NextAction"]:
        self.entryBoxes[header].bind("<Tab>", self.convertDate)

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

    self.backupButton = tk.Button(self.adminButtonFrame,
                                  text="Backup database",
                                  command = lambda : self.notify(self.db.backup()))
    self.backupButton.grid(sticky="W")

    self.refreshButton = tk.Button(self.adminButtonFrame,
                                       text="Refresh",
                                       command = self.refreshAll)
    self.refreshButton.grid(sticky="W")

  # todo Add ability to mark certain days to not work on certain categories (ie. no school on weekends!) This would also make sense in a second database, and would give a better idea of actual workloads by not falsely assuming I'm gonna do as much homework on weekends as weekdays. Easiest would be to just mark days of the week (weekend/weekday), more complicated would be individual days. Best might be a combination?
  # todo put the next action / due date at a specific time?
  # todo Track hours worked per day (link timer and calendar, probably in a second database)
  # todo add buttons to scroll the calendar forward week-by-week
  # todo what if clicking on a day in the calendar would show all the tasks for that day in the task list
  # eg. thisDay["LoadLabel"].bind("<Button-1>", CALLBACK)
  # Set up the calendar display to show estimated workload each day for a several week forecast
  def setupCalendar(self):
    self.numweeks = 4

    #Build the calendar out of labels
    self.calendar = []

    #Add day of week names at top, but won't change so don't save
    for i, day in enumerate(["Mon", "Tue", "Wed", "Thu", "Fri"]):
      tk.Label(self.calendarFrame, font=self.font + ("bold",), text=day).grid(row=0, column=i, padx=4, pady=4)

    for week in range(self.numweeks):
      thisWeek = []
      for day in range(5):
        thisDay = {}
        # todo *Sometimes* this significantly slows boot time. Could maybe cut down on labels by having dates all in a row for each week, but lining up with loads could be tricky. First row changes colour, so could do each date row below the first as a multi-column label.
        #Alternate date labels and workloads
        thisDay["DateLabel"] = tk.Label(self.calendarFrame, font=self.font)
        thisDay["DateLabel"].grid(row=2*week + 1, column=day, padx=4, pady=4)
        thisDay["LoadLabel"] = tk.Label(self.calendarFrame, font=self.font)
        thisDay["LoadLabel"].grid(row=2*week + 2, column=day, padx=4, pady=4)
        thisWeek.append(thisDay)
      self.calendar.append(thisWeek)

    self.updateCalendar()

  def setupKeybindings(self):
    # These aren't all the keybindings, but they're all the ones the user should notice
    # Other keybindings mostly just make the app behave how you'd expect
    self.root.bind("<Control-s>", lambda event: self.backupButton.invoke())
    self.root.bind("<Control-q>", lambda event: self.root.destroy())
    self.root.bind("<Control-w>", lambda event: self.root.destroy())
    self.root.bind("<Control-n>", lambda event: self.newTaskButton.invoke())
    self.root.bind("<Control-r>", lambda event: self.refreshButton.invoke())
    self.root.bind("<Control-f>", lambda event: self.searchBox.focus())

  ######################################################
  # GUI update functions

  def updateCalendar(self):

    self.db.calculateDayLoads(self.numweeks)

    #calendar.weekheader(3) prints Mon-Fri
    #calendar.month(YYYY, month, width, height) prints calendar
    today = datetime.datetime.today()
    thisMonday = today - datetime.timedelta(days=today.weekday())
    self.calendarDays = []
    for week in range(self.numweeks):
      thisWeek = []
      for day in range(5):
        thisDate = thisMonday + datetime.timedelta(days=day, weeks=week)
        thisWeek.append(thisDate)
        thisDay = self.calendar[week][day]
        thisDay["DateLabel"].config(text=thisDate.strftime("%b %d"))
        if thisDate == today:
          thisDay["DateLabel"].config(bg="lime")
        else:
          thisDay["DateLabel"].config(bg="#d9d9d9")
        if thisDate >= today:
          hoursThisDay = self.db.getDayTotalLoad(thisDate.strftime("%Y-%m-%d")) / 60
          thisDay["LoadLabel"].config(text=str(round(hoursThisDay,1)),
                                      bg=greenRedScale(0,7,hoursThisDay))
        else:
          thisDay["LoadLabel"].config(text="", bg="#d9d9d9")
      self.calendarDays.append(thisWeek)

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

  #Set up filters to match the selected task (ideally catching all of a repeating task)
  def multiEditConfig(self):
    if self.multiEdit.get():
      #Enter special mode
      self.refreshButton.config(state="disabled")
      self.timeButton.config(state="disabled")
      self.entryBoxes["Used"].config(state="readonly")

      if self.selection is not None:
        #Enter multiedit mode
        self.previousSearch = self.searchBox.get()
        self.overwriteEntryBox(self.searchBox, self.loadedTasks[self.selection]["Task"])
        self.catBox.set(self.loadedTasks[self.selection]["Category"])
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
      self.refreshButton.config(state="normal")
      self.timeButton.config(state="normal")
      self.entryBoxes["Used"].config(state="normal")

      if self.selection is not None:
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


  #Pretty self explanatory. Used because select_set doesn't trigger <<ListboxSelect>> event
  def selectListboxItem(self, item):
    self.lb.select_set(item)
    self.onSelect()

  #When the listbox gains focus, select the first item
  def selectFirst(self, event=tk.Event):
    if self.selection is None:
      self.selectListboxItem(0)

  # todo timer should probably be its own class
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
    FMT = "%H:%M:%S"
    if self.timing:
      runTime = (datetime.datetime.strptime(time.strftime(FMT), FMT)
                 - datetime.datetime.strptime(self.startTime, FMT))
      # If the timer is run through midnight it goes negative. This fixes it.
      if runTime.days < 0:
        runTime = runTime + datetime.timedelta(days=1)

      try:
        if self.selection is None:
          raise ValueError("Cannot time an empty task")
        self.timerVal = (runTime
                         + datetime.timedelta(minutes=(self.loadedTasks[self.selection]["Used"] or 0)))
        self.timeLabel.config(text=str(self.timerVal))
        self.root.after(1000, self.runTimer)
      except ValueError as e:
        self.notify(e)
        self.timerVal = None
        self.timeButton.invoke()

  def convertDate(self, event=tk.Event):
    box = event.widget
    dateStr = box.get()
    convertedDate = ""

    try:
      datetime.datetime.strptime(dateStr, '%Y-%m-%d')
      convertedDate = dateStr
    except ValueError:
      try:
        #eg. Jan 1, 21
        convertedDate = datetime.datetime.strptime(dateStr, "%b %d, %y").strftime('%Y-%m-%d')
      except ValueError:
        #Date string doesn't match
        try:
          #Try to add the current year
          #eg. Jan 1
          convertedDate = datetime.datetime.strptime(dateStr, "%b %d").replace(year = datetime.datetime.today().year).strftime('%Y-%m-%d')
        except ValueError:
          #Date really doesn't match
          self.notify("Can't match date format of {}".format(dateStr))
          return

    box.delete(0, tk.END)
    box.insert(0, convertedDate)

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

  def refreshFilterCategories(self):
    self.filterCategories = list(set([task["Category"] for task in self.loadedTasks]))
    self.filterCategories.sort()
    self.filterCategories = ["All categories"] + self.filterCategories
    self.catBox.config(values=self.filterCategories)

  #Updates the categories in the category filter
  def refreshCategories(self):
    self.categories = self.db.getCategories()
    try:
      self.entryBoxes["Category"].config(values=self.categories)
    except AttributeError:
      #Fails on setup
      pass

  #Updates the listbox with the loaded tasks
  def showTasks(self):
    #Max length of any item in each column, including headers
    self.maxlens = [max([len(str(row[column])) for row in self.loadedTasks] + [len(column)]) for column in self.displayColumns]
    #Forces these two to 50 to try and maintain some order
    self.maxlens[self.displayColumns.index("Task")] = 60
    self.maxlens[self.displayColumns.index("Notes")] = 59

    #Adjust the listbox to be wide enough
    self.lb.config(width=sum(self.maxlens) + len(self.maxlens) * 3)

    self.recordLabel.config(text=str(len(self.loadedTasks)) + " tasks found")

    #Create the header text
    headerline = ""
    for header, length in zip(self.displayColumns, self.maxlens):
      headerline += header.center(length) + " | "
    headerline = headerline[:-2]
    self.headerLabel.config(text = headerline)

    #delete tasks and reinsert
    self.lb.delete(0,'end')
    for task in self.loadedTasks:
      line = ""
      for (header,length) in zip(self.displayColumns, self.maxlens):
        try:
          line += datetime.datetime.strptime(str(task[header]), '%Y-%m-%d').strftime("%b %d, %y").ljust(length) + " | "
        except ValueError:
          line += ljusttrunc(str(task[header]), length) + " | "
      line = line[:-2]

      self.lb.insert(tk.END,line)
      #Colour-coding!
      try:
        self.lb.itemconfig(tk.END, {'bg': greenRedScale(0, 60, task["Load"])})
      except TypeError:
        #Fails for items where Load is None, eg. completed, not yet active
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
          criterion = datetime.date.today().strftime("%Y-%m-%d")
          quote     = "'"
        else:
          criterion = filterBox.get()
          quote     = "'"

      if criterion is not None:
        criteria.append(header + operator + surround(escapeSingleQuotes(criterion), quote))

    return criteria

  def refreshTasks(self, event=tk.Event):
    #Remember which task was selected
    if self.selection not in [None, -1]:
      self.selected_rowid = self.loadedTasks[self.selection]["rowid"]

    criteria = self.getSearchCriteria()
    self.loadTasks(criteria)
    self.showTasks()

    if self.selection not in [None, -1]:
      #Find the previously selected task
      previousSelection = None
      for i, task in enumerate(self.loadedTasks):
        if task["rowid"] == self.selected_rowid:
          previousSelection = i
          break

      self.selection = previousSelection

      if previousSelection is not None:
        self.selectListboxItem(self.selection)
      else:
        self.clearEntryBoxes()

    #Update the category filterbox to only show available categories
    self.refreshFilterCategories()

  def resetListboxSelection(self):
    self.lb.selection_clear(0,'end')
    self.lb.select_set(self.selection)

  def confirmDiscardChanges(self):
    if self.nonTrivialChanges():
      selection = tk.messagebox.askyesnocancel(title="Save before switching?",
                                               message = "Do you want to save your changes to '{}' before switching?".format(self.loadedTasks[self.selection]["Task"]))
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
                                                 message="Do you want to save the timer for '{}' before switching?".format(self.loadedTasks[self.selection]["Task"]))
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
    self.selection = None

  #Lets you scroll with arrow keys
  def onDown(self, event):
    if self.selection < self.lb.size() - 1:
      self.lb.select_clear(self.selection)
      self.selectListboxItem(self.selection + 1)

  #Lets you scroll with arrow keys
  def onUp(self, event):
    if self.selection > 0:
      self.lb.select_clear(self.selection)
      self.selectListboxItem(self.selection - 1)

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
      if self.selection != self.lb.curselection()[0]:
        if self.confirmDiscardChanges():
          if self.confirmCancelTimer():
            self.selection = self.lb.curselection()[0]
          else:
            raise PermissionError

          #todo this could be a function "update entryBoxes" or something
          for (header, entry) in [(header, self.entryBoxes[header]) for header in self.editColumns]:
            if header == "Flex":
              entry.set(self.loadedTasks[self.selection][header])
            else:
              self.overwriteEntryBox(entry, self.loadedTasks[self.selection][header])

          self.checkDone.set(self.loadedTasks[self.selection]["O"])
      if not self.timing:
        self.timeLabel.config(text=str(datetime.timedelta(minutes=(self.loadedTasks[self.selection]["Used"] or 0))))

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
    self.updateCalendar()
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
            datetime.datetime.strptime(data, '%Y-%m-%d')
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
    today = datetime.date.today().strftime("%Y-%m-%d")

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
    taskToDelete = self.loadedTasks[self.selection]
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
        self.selection = -1

    except TypeError:
      self.notify("Cannot delete - none selected")

  #Save the current state of the entry boxes for that task
  def save(self, event=tk.Event()):
    if self.confirmCancelTimer():
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
        self.notify(e)
      except PermissionError as e:
        #updateSelectedTask() cancelled by user
        self.notify(e)

  # todo a more elegant way of handling repeating tasks than just creating a bunch of duplicates
  def createTaskFromInputs(self):
    newRowDict = {}

    #Pull in directly entered values
    for header in self.editColumns:
      newRowDict[header] = self.getEntry(self.entryBoxes[header])

    #Store original values
    newRowDict["Budget"] = newRowDict["Time"]
    newRowDict["StartDate"] = newRowDict["NextAction"]
    newRowDict["DateAdded"] = datetime.date.today().strftime("%Y-%m-%d")
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
        thisRowDict[header] = datetime.datetime.strftime(datetime.datetime.strptime(thisRowDict[header], "%Y-%m-%d") + i * interval, "%Y-%m-%d")

      thisRowDict = self.calculateRow(thisRowDict)
      self.validateRow(thisRowDict)

      headers = [h for h in self.db.headers if h != "rowid"]
      vals = [thisRowDict[header] for header in headers]

      self.db.createTask(headers, vals)

    self.db.commit()
    # This is so you don't accidentally create multiple of the same task by clicking save multiple times
    # todo would be more elegant to just select the newly created task
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
    if self.selection in [None, -1]:
      pass
    else:
      #Find which columns were changed and how

      newRow = {}
      oldRow = dict(self.loadedTasks[self.selection])

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

  # todo would be nice if multiupdate could change names with a delta, not just dates UPDATE (REPLACE(str,search,replacement)?)
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
        criteria = ["rowid = {}".format(self.loadedTasks[self.selection]["rowid"])]
        # todo messy
        # Dump the time worked to external time tracker
        for change in changes:
            if change.find("Used") != -1:
                timediff = int(re.findall(r"(\d+)", change)[0])
                with open("timesheet.csv", "a") as f:
                    f.write("{}, {}, {}, {}\n".format(datetime.datetime.today().strftime("%Y-%m-%d"), self.loadedTasks[self.selection]["Category"], timediff, self.loadedTasks[self.selection]["Task"]))

      self.db.updateTasks(criteria, changes)

      self.db.commit()

      if self.multiEdit.get():
        #So the new search still includes the task, even if you change the name or category
        self.overwriteEntryBox(self.searchBox, self.getEntry(self.entryBoxes["Task"]))
        self.catBox.set(self.getEntry(self.entryBoxes["Category"]))

  def duplicateTask(self):

    oldSelection = self.selection
    self.selection = None
    self.save()
    self.selectListboxItem(oldSelection)
    self.notify("Duplicated task")

  #scans all tasks and updates using calculateRow()
  def updateLoadsToday(self, event=tk.Event):
    try:
      #backup task list
      oldTasks = self.loadedTasks
    except AttributeError:
      pass

    today = datetime.datetime.today().strftime("%Y-%m-%d")
    self.loadTasks(["O == 'O'","NextAction <= '{}'".format(today)])

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
    self.connect(databasePath)
    self.databasePath = databasePath

  def connect(self, databasePath):
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

  def backup(self):
    #Strips '.db', inserts the -date and re-adds .db
    path = self.databasePath[:-3] + "-" + str(datetime.date.today()) + ".db"
    # Different system call for linux vs. Windows. Never tried running on Mac, but the first would probably work b/c *nix
    if sys.platform == "linux":
      subprocess.run(["cp", self.databasePath, path])
    else:
      subprocess.run(["copy", self.databasePath, path], shell=True)
    return "Database backed up to: {}".format(path)

  # TODO
  # Iterate over every task, with a start date from now to the end of the week self.numweeks from now, in order of due date, from soonest due to latest
  # For each task, distribute time evenly across its open period. If a day hits 8 hours, do not add more time.
  def calculateDayLoads(self, numweeks):
    # Get a list of all unfinished tasks with start dates no more than self.numweeks in the future, sorted from soonest due date to latest
    today = datetime.datetime.today()
    thisFriday = today - datetime.timedelta(days=today.weekday() + 4)
    lastRenderedDate = thisFriday + datetime.timedelta(weeks=numweeks-1)
    self.cwrite.execute("SELECT NextAction, DueDate, Left FROM worklist WHERE O == 'O' AND NextAction <= ? ORDER BY DueDate;", [lastRenderedDate.strftime("%Y-%m-%d")])
    relevantTasks = self.cwrite.fetchall()
  
    # Iterate over the list of tasks (starting from soonest due date), distributing time evenly (each day gets time remaining / # days remaining) over days from max(today, start date) to due date. If adding time would push day over 8 hours, only add up to 8 hours, and withold extra time within the task. 
    self.dayLoads = {}
    for task in relevantTasks:
      # todo around here would be a decent place to do recursion
      remainingLoad = task["Left"]
      #TODO datetime.datetime.strptime(var, "%Y-%m-%d") needs its own function, this is getting ridiculous
      #TODO also var.strftime("%Y-%m-%d)
      startDate = max(today, datetime.datetime.strptime(task["NextAction"], "%Y-%m-%d"))
      dateRange = [startDate + datetime.timedelta(days=n) for n in range(0, daysBetween(startDate.strftime("%Y-%m-%d"), task["DueDate"]) + 1)]

      for thisDay in dateRange:
        if np.is_busday(thisDay.date()):
          # TODO This needs to change once the overflow code down below is fixed. This backloads time by squishing extra time away, rather than distributing evenly or optimally
          loadDeposit = remainingLoad / workDaysBetween(thisDay.date(), task["DueDate"])
          # Do not push a day over 8 hours
          try:
              loadDeposit = min(max(8*60 - self.dayLoads[thisDay.strftime("%Y-%m-%d")], 0), loadDeposit)
              self.dayLoads[thisDay.strftime("%Y-%m-%d")] += loadDeposit
          except KeyError:
              # If this day has no load assigned to it yet, there will not be an entry in the dict and a key error will occur
              loadDeposit = min(8*60, loadDeposit)
              self.dayLoads[thisDay.strftime("%Y-%m-%d")] = loadDeposit
  
          remainingLoad -= loadDeposit

        # TODO placeholder until we have a better way to deal with overflow (see TODO below)
        # TODO If time remains (i.e. one or more days was maxed out to 8 hours), distribute remaining time evenly over all tasks (TODO: doing it recursively, noting the number of days maxed out and using a new quotient to calculate average load each time would be better, although you would need an end condition other than "all time distributed" since it's not guaranteed that all days can be kept to 8 hours or less with this method).
          if thisDay.strftime("%Y-%m-%d") == task["DueDate"] and remainingLoad != 0:
            self.dayLoads[thisDay.strftime("%Y-%m-%d")] += remainingLoad
            remainingLoad = 0 # unecessary but comforts me

  # Gets the work load for the day represented by the passed string
  #date should be a string formatted "YYYY-MM-DD"
  def getDayTotalLoad(self, date):
    # Will raise an error if date is poorly formatted
    datetime.datetime.strptime(date, '%Y-%m-%d')

    return self.dayLoads[date]

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

def main():
  if len(sys.argv) > 1:
    taskWindow = TaskListWindow(sys.argv[1])
    taskWindow.runLoop()
  elif os.path.isfile("worklist.db"):
    #default
    taskWindow = TaskListWindow("worklist.db")
    taskWindow.runLoop()
  else:
    print("No worklist found and none specified.\nCreating new worklist.db")
    conn = sqlite3.connect("worklist.db")
    cur  = conn.cursor()
    # todo a better name for "Load" would be "CurrentLoad"
    cur.execute("CREATE TABLE worklist('Category' TEXT,'O' TEXT,'Task' TEXT,'Budget' INTEGER,'Time' INTEGER,'Used' INTEGER,'Left' INTEGER,'StartDate' TEXT,'NextAction' TEXT,'DueDate' TEXT,'Flex' TEXT,'DaysLeft' INTEGER,'TotalLoad' REAL,'Load' REAL,'Notes' TEXT,'DateAdded' TEXT)")
    cur.close()
    taskWindow = TaskListWindow("worklist.db")
    taskWindow.runLoop()

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
  d1 = datetime.datetime.strptime(d1, "%Y-%m-%d")
  d2 = datetime.datetime.strptime(d2, "%Y-%m-%d")
  return (d2 - d1).days

# takes strings "%Y-%m-%d"
# inclusive of start and end date
def workDaysBetween(d1, d2):
  return int(np.busday_count(d1, (datetime.datetime.strptime(d2, "%Y-%m-%d") + datetime.timedelta(days=1)).date()))

if __name__ == '__main__':
  main()
