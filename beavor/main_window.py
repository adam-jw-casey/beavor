import tkinter as tk

from .backend import Project, Deliverable
from .ScrollFrame import ScrollFrame

from typing import Optional

class ProjectWindow(ScrollFrame):
    def __init__(self, parent):
        super().__init__(parent, "No project selected")
        self.deliverable_rows: list[DeliverableRow] = []
        self.select_project(None)

    def select_project(self, proj: Optional[Project]):
        self.selected_project = proj
        self.config(text=proj.name if proj else "No project selected")

        for _ in range(len(self.deliverable_rows)):
            self.deliverable_rows.pop().destroy()

        if proj is None:
            return

        for deliverable in sorted(proj.deliverables, key=lambda d: d.due):
            self.add_deliverable_row(deliverable)

    def add_deliverable_row(self, deliverable: Deliverable):
        deliverable_row = DeliverableRow(self.viewPort, deliverable)
        deliverable_row.grid(sticky=tk.W + tk.E)
        self.deliverable_rows.append(deliverable_row)

class DeliverableRow(tk.LabelFrame):
    def __init__(self, parent, deliverable: Deliverable):
        super().__init__(parent, text=deliverable.name)
        self.notes_label=tk.Label(self, text=deliverable.notes)
        self.notes_label.grid()
