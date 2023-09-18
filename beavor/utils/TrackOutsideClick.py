import tkinter as tk
from typing import Callable, Optional

class TrackOutsideClick:
    """
    Register when click events occur outside of a given widget
    """
    def __init__(self, domain: list[tk.Widget]):
        self.funcid1: str
        self.funcid2: str
        self.funcid3: str
        self.funcid4: str

        self.domain: list[tk.Widget] = domain
        self.bound: bool = False

    def bind(self, callback: Callable[[], Optional[str]], ignore_self_click=False):
        # Only allow a single binding, to avoid losing ids of old bindings
        # and leaving them dangling
        if self.bound:
            self.unbind()
        self.bound = True

        # Optionally ignore clicks that occur on domain widget
        wrapped_callback: Callable[[tk.Event], Optional[str]]
        if ignore_self_click:
            wrapped_callback = lambda e: (callback() if e.widget not in self.domain else None)
        else:
            wrapped_callback = lambda _: callback()

        self.funcid1 = self.domain[0].winfo_toplevel().bind("<1>", wrapped_callback, "+")
        self.funcid2 = self.domain[0].winfo_toplevel().bind("<2>", wrapped_callback, "+")
        self.funcid3 = self.domain[0].winfo_toplevel().bind("<3>", wrapped_callback, "+")
        self.funcid4 = self.domain[0].winfo_toplevel().bind("<Configure>", lambda e: (callback() if e.widget == self.domain[0].winfo_toplevel() else None), "+")

    def unbind(self):
        self.domain[0].winfo_toplevel().unbind("<1>", self.funcid1)
        self.domain[0].winfo_toplevel().unbind("<2>", self.funcid2)
        self.domain[0].winfo_toplevel().unbind("<3>", self.funcid3)
        self.domain[0].winfo_toplevel().unbind("<Configure>", self.funcid4)

        self.bound = False
