import tkinter as tk
from typing import List

from .ScrollFrame import ScrollFrame
from .backend import Category, Project

class CategoryScroller(ScrollFrame):
    def __init__(self, parent, onRowClick):
        super().__init__(parent, "Projects")
        self.categoryRows: list[CategoryRow] = []
        self.onRowClick = onRowClick
        self.grid_columnconfigure(0, weight=1)
    
    def showCategories(self, categories: List[Category]):
        for _ in range(len(self.categoryRows)):
            self.categoryRows.pop().destroy()
            
        for category in categories:
            self.add_category_row(category)

    def add_category_row(self, category):
        categoryRow = CategoryRow(self.viewPort, category, lambda c=category.name: self.onRowClick(c))
        categoryRow.grid(sticky=tk.W + tk.E)
        self.categoryRows.append(categoryRow)

class CategoryRow(tk.Frame):
    def __init__(self, parent: tk.Frame, category: Category, select_project):
        def on_project_click(proj: Project):
            select_project(proj)
            self.unhighlight_all()

            next(filter(lambda pr: pr.project == proj, self.project_rows)).highlight()

        super().__init__(parent)

        self.category_name = category.name
        self.nameLabel = tk.Label(self, text=('▸ ' if len(category.projects) > 0 else '   ') + self.category_name)
        self.nameLabel.grid(sticky=tk.W + tk.E)
        self.nameLabel.bind("<1>", lambda _: self.on_click())

        self.expanded = False

        self.on_project_click = on_project_click

        self.project_rows: list[ProjectRow] = []
        for proj in category.projects:
            self.add_project_row(proj)

    def add_project_row(self, proj: Project):
            pr = ProjectRow(self, proj, self.on_project_click)
            self.project_rows.append(pr)
            pr.grid(row=len(self.project_rows), sticky=tk.W)
            pr.grid_forget()

    def expand(self):
        self.nameLabel.configure(text= '▾ ' + self.category_name)
        for pr in self.project_rows:
            pr.grid()

        self.expanded = True

    def collapse(self):
        self.nameLabel.configure(text= '▸ ' + self.category_name)
        for pr in self.project_rows:
            pr.unhighlight()
            pr.grid_forget()

        self.expanded = False

    def on_click(self):
        if len(self.project_rows) == 0:
            return

        if self.expanded:
            self.collapse()
        else:
            self.expand()

    def unhighlight_all(self):
        for pr in self.project_rows:
            pr.unhighlight()

class ProjectRow(tk.Frame):
    def __init__(self, parent: tk.Frame, project: Project, callback):
        super().__init__(parent)

        self.project = project

        self.nameLabel = tk.Label(self, text="-- " + project.name)
        self.nameLabel.grid(sticky=tk.W)
        self.nameLabel.bind("<1>", lambda _: callback(self.project))

        self.visible = [self, self.nameLabel]

    def highlight(self) -> None:
        for w in self.visible:
            w.config(bg="lightblue")

    def unhighlight(self) -> None:
        for w in self.visible:
            w.config(bg="white")
