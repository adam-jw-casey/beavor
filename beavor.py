#!/usr/bin/python3.11

import sys, os
from widgets import WorklistWindow
from utils import DatabaseManager

DEFAULT_DATABASE_PATH = "worklist.db"

def main():
  worklist: WorklistWindow
  if len(sys.argv) > 1:
    worklist = WorklistWindow(sys.argv[1])
  elif os.path.isfile(DEFAULT_DATABASE_PATH):
    #default
    worklist = WorklistWindow(DEFAULT_DATABASE_PATH)
  else:
    print("No worklist found and none specified.\nCreating new {DEFAULT_DATABASE_PATH}")
    DatabaseManager.createNewDatabase(DEFAULT_DATABASE_PATH)
    worklist = WorklistWindow(DEFAULT_DATABASE_PATH)

  worklist.root.mainloop()

if __name__ == "__main__":
  main()
