import sys
import tkinter as tk

from typing import Optional

from .main_window import ProjectWindow
from .sidebar import CategoryScroller
from .backend import DatabaseManager, Project, Category

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

        # Make main window that shows the selected project
        self.root.grid_columnconfigure(1, weight=1)
        self.main_window = ProjectWindow(self.root)
        self.main_window.grid(row=0, column=1, sticky=tk.N+tk.S+tk.E+tk.W)

        # Make sidebar that displays categories and projects
        self.sidebar = CategoryScroller(self.root, self.select_project, self.create_category, self.rename_category)
        self.sidebar.grid(row=0, column=0, sticky = tk.N+tk.S+tk.E+tk.W)
        self.sidebar.showCategories(self.db.get_all())

    ##########
    # Events #
    ##########

    def select_project(self, proj: Optional[Project]):
        """
        Update any widget that needs to respond to the selected category changing
        """
        self.main_window.select_project(proj)

    def create_category(self):
        self.db.create_default_category()
        self.sidebar.showCategories(self.db.get_all())

    def rename_category(self, category: Category, new_name: str):
        category.name = new_name
        self.db.update_category(category)
