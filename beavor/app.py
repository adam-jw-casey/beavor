import sys
import tkinter as tk

from .widgets import CategoryScroller
from .backend import DatabaseManager, Category

class WorklistWindow():
    def __init__(self, databasePath: str):
        self.os = sys.platform

        self.db = DatabaseManager(databasePath)

        self.root = tk.Tk()
        self.root.option_add("*background", "white")
        self.root.configure(bg="white")

        # OS-dependent settings
        if self.os == "linux":
          self.root.attributes('-zoomed', True)
          self.root.option_add("*Font", "TkFixedFont 12")
        else:
          #win32
          self.root.state("zoomed")
          self.root.option_add("*Font", "Courier 10")

        # Add window icon
        self.root.winfo_toplevel().title("Beavor")
        self.root.iconphoto(False, tk.PhotoImage(file="./.resources/beavor.png"))

        # Make main row expand to take up room
        self.root.grid_rowconfigure(0, weight=1)

        # Make sidebar that displays categories and projects
        self.sidebar = tk.LabelFrame(self.root, text="Projects")
        self.sidebar.grid(row=0, column=0, sticky = tk.N+tk.S)
        self.sidebar.grid_rowconfigure(0, weight=1)

        self.category_scroller = CategoryScroller(self.sidebar, lambda s: print(s.name))
        self.category_scroller.grid(row=0, column=0, sticky = tk.N + tk.S)
        self.category_scroller.showCategories(self.db.get_all())

        # Make main window that shows the selected project
        self.root.grid_columnconfigure(1, weight=1)
        self.main_window = tk.LabelFrame(self.root, text="No project selected")
        self.main_window.grid(row=0, column=1, sticky=tk.N+tk.S+tk.E+tk.W)
