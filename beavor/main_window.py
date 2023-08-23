import tkinter as tk

from .backend import Project, Deliverable
from .ScrollFrame import ScrollFrame
from .widgets import LabeledWidget, EditableLabel

from typing import Optional

class ProjectWindow(ScrollFrame):
    def __init__(self, parent, update_deliverable):
        super().__init__(parent, "No project selected")
        self.deliverable_rows: list[DeliverableRow] = []
        self.select_project(None)

        self.update_deliverable = update_deliverable

    def select_project(self, proj: Optional[Project]):
        self.selected_project = proj
        self.config(text=proj.name if proj else "No project selected")

        for _ in range(len(self.deliverable_rows)):
            self.deliverable_rows.pop().destroy()

        if proj is None:
            return

        proj.deliverables.sort(key=lambda d: d.due)
        for deliverable in proj.deliverables:
            self.add_deliverable_row(deliverable)

    def add_deliverable_row(self, deliverable: Deliverable):
        deliverable_row = DeliverableRow(self.viewPort, deliverable, lambda s, d=deliverable: self.update_deliverable_notes(d, s))
        deliverable_row.grid(sticky=tk.W + tk.E)
        self.deliverable_rows.append(deliverable_row)

    def update_deliverable_notes(self, deliverable: Deliverable, s: str):
        deliverable.notes = s
        self.update_deliverable(deliverable)

class DeliverableRow(tk.LabelFrame):
    def __init__(self, parent, deliverable: Deliverable, update_notes):
        super().__init__(parent, text=deliverable.name)

        self.due_label = LabeledWidget(self, "📅 ", lambda p: tk.Label(p, text=deliverable.due))
        self.due_label.grid(sticky=tk.W)

        self.notes_label = LabeledWidget(self, "🖉 ", lambda p: EditableLabel(p, deliverable.notes, update_notes))
        self.notes_label.grid(sticky=tk.W)
