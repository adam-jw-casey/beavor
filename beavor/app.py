import sys
import tkinter as tk

from .widgets import CategoryScroller
from .backend import DatabaseManager, Category

class WorklistWindow():
    def __init__(self, databasePath: str):
        self.os = sys.platform

        self.db = DatabaseManager(databasePath)

        self.root = tk.Tk()

        # OS-dependent settings
        if self.os == "linux":
          self.root.attributes('-zoomed', True)
          self.font = ("Liberation Mono", 10)
        else:
          #win32
          self.root.state("zoomed")
          self.font = ("Courier", 10)

        # Add window icon
        self.root.winfo_toplevel().title("Beavor")
        self.root.iconphoto(False, tk.PhotoImage(file="./.resources/beavor.png"))

        # Make main row expand to take up room
        self.root.grid_rowconfigure(0, weight=1)

        self.sidebar = tk.LabelFrame(self.root, text="Projects")
        self.sidebar.grid_rowconfigure(0, weight=1)
        self.sidebar.grid(row=0, column=0, sticky = tk.N+tk.S)

        self.category_scroller = CategoryScroller(self.sidebar, lambda s: print(s.name))
        self.category_scroller.grid(row=0, column=0, sticky = tk.N + tk.S)
        self.category_scroller.showCategories(self.db.get_all())
