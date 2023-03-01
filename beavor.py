#!/usr/bin/python3.11

import sys, os
import sqlite3
from widgets import WorklistWindow

def main():
  worklist: WorklistWindow
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
    cur.execute("""
        CREATE TABLE worklist(
            'Category'  TEXT,
            'O'         TEXT,
            'Task'      TEXT,
            'Budget'    INTEGER,
            'Time'      INTEGER,
            'Used'      INTEGER,
            'Left'      INTEGER,
            'StartDate' TEXT,
            'NextAction'TEXT,
            'DueDate'   TEXT,
            'Flex'      TEXT,
            'DaysLeft'  INTEGER,
            'TotalLoad' REAL,
            'Load'      REAL,
            'Notes'     TEXT,
            'DateAdded' TEXT)
    """)
    cur.close()
    worklist = WorklistWindow("worklist.db")
  worklist.root.mainloop()

if __name__ == "__main__":
  main()
