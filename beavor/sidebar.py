import tkinter as tk
from tktooltip import ToolTip

from typing import List

from .ScrollFrame import ScrollFrame
from .backend import Category, Project
from .widgets import ContextMenuSpawner, EditableLabel

class CategoryScroller(ScrollFrame):
    def __init__(self, parent, onRowClick, create_category, rename_category, delete_category):
        def context_menu_builder() -> tk.Menu:
            ctx = tk.Menu(self, tearoff=0)
            ctx.add_command(label="New category", command=create_category)

            return ctx

        super().__init__(parent, "Projects")
        self.categoryRows: list[CategoryRow] = []
        self.ctx = ContextMenuSpawner([self, self.canvas], context_menu_builder)

        self.onRowClick = onRowClick
        self.rename_category = rename_category
        self.delete_category = delete_category

    def showCategories(self, categories: List[Category]):

        for _ in range(len(self.categoryRows)):
            self.categoryRows.pop().destroy()
            
        for category in categories:
            categoryRow = CategoryRow(
                self.viewPort,
                category,
                lambda c=category.name: self.onRowClick(c),
                lambda new_name, cat=category: self.rename_category(cat, new_name),
                lambda c=category: self.delete_category(c)
            )
            categoryRow.grid(sticky=tk.W + tk.E)
            self.categoryRows.append(categoryRow)

class CategoryRow(tk.Frame):
    def __init__(self, parent: tk.Frame, category: Category, select_project, update_category_name, delete_category):
        def on_project_click(proj: Project):
            select_project(proj)
            self.unhighlight_all()

            next(filter(lambda pr: pr.project == proj, self.project_rows)).highlight()

        def get_tooltip() -> str:
            if len(self.project_rows) == 0:
                return "No projects in this category"
            elif self.expanded:
                return "Click to hide projects"
            else:
                return "Click to show projects"

        def context_menu_builder() -> tk.Menu:
            ctx = tk.Menu(self, tearoff=0)
            ctx.add_command(label="Delete Category", command=delete_category)

            return ctx

        super().__init__(parent)
        self.grid_columnconfigure(1, weight=1)

        self.category_name = category.name
        self.select_none = lambda: select_project(None)

        self.icon = tk.Label(self, text=('▸ ' if len(category.projects) > 0 else '   ') + '📁')
        self.icon.grid(row=0, column=0, sticky=tk.W)
        self.icon.bind("<Button-1>", lambda _: self.on_click())
        ToolTip(self.icon, msg=get_tooltip, delay=0.3)

        self.nameLabel = EditableLabel(self, text=self.category_name, edit_text=update_category_name)
        self.nameLabel.grid(row=0, column=1, sticky=tk.W+tk.E)
        self.ctx = ContextMenuSpawner([self.nameLabel], context_menu_builder)

        self.expanded = False

        self.on_project_click = on_project_click

        self.project_rows: list[ProjectRow] = []
        for proj in category.projects:
            self.add_project_row(proj)

    def add_project_row(self, proj: Project):
            pr = ProjectRow(self, proj, self.on_project_click, prefix="     💡 " )
            self.project_rows.append(pr)
            pr.grid(row=len(self.project_rows), column=0, sticky=tk.W+tk.E, columnspan=2)
            pr.grid_forget()

    def expand(self):
        self.icon.configure(text= '▾ 📁')
        for pr in self.project_rows:
            pr.grid(sticky=tk.W+tk.E, columnspan=2)

        self.expanded = True

    def collapse(self):
        self.icon.configure(text= '▸ 📁')
        for pr in self.project_rows:
            pr.unhighlight()
            pr.grid_forget()

        self.expanded = False

        self.select_none()

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

class ProjectRow(tk.Label):
    def __init__(self, parent: tk.Frame, project: Project, callback, prefix: str):
        self.project = project

        super().__init__(parent, text=prefix+ project.name, anchor=tk.W)
        self.bind("<1>", lambda _: callback(self.project))
        self.bind("<Enter>", lambda _: self.on_mouseover())
        self.bind("<Leave>", lambda _: self.on_mouseleave())

        self.highlighted = False

    def highlight(self) -> None:
        self.config(bg="lightblue")
        self.highlighted = True

    def unhighlight(self) -> None:
        self.config(bg="white")
        self.highlighted = False

    def on_mouseover(self):
        if not self.highlighted:
            self.config(bg="lightgrey")

    def on_mouseleave(self):
        if not self.highlighted:
            self.unhighlight()
