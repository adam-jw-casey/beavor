import tkinter as tk
import tkinter.ttk as ttk
from typing import List

class CompletingComboBox(ttk.Combobox):
    def __init__(self, parent, getOptions):
        super().__init__(parent)

        self.bind("<FocusOut>", lambda _: self.selection_clear())
        self.bind("<KeyRelease>", lambda event: self._completeBox(event, getOptions))
        self.bind("<Return>", lambda _: self.icursor(tk.END))

        self.config(values=getOptions())

    def _completeBox(self, event: tk.Event, getSourceList) -> None:

      #Don't run when deleting, or when shift is released
      if event.keysym in ["BackSpace", "Shift_L", "Shift_R"]:
          return

      cursorPos: int = self.index(tk.INSERT)
      current: str = self.get()[:]

      #Don't run if self is empty, or cursor is not at the end
      if current and cursorPos == len(self.get()):
        # Find all options beginning with the current string
        options: List[str] = list(filter(lambda s: s.find(current) == 0, getSourceList()))

        if options:
            # Find longest shared leading (from cursor) substring among matching options
            i: int = len(current)-1
            while i < min([len(o) for o in options]):
                if len(set([option[i] for option in options])) != 1:
                    break
                i += 1

            # If found a match
            if i > len(current):
              self.insert(tk.END, options[0][cursorPos:i+1])

            self.select_range(cursorPos, tk.END)
            self.icursor(tk.END)

