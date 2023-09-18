import tkinter as tk
from typing import Callable
from .TrackOutsideClick import TrackOutsideClick

class ContextMenuSpawner:
    """
    Spawns tk.Menus with a given configuration on right click on parent

    This CANNOT inherit from tk.Menu, even thought it feels like it should,
    because it needs to be able to spawn multiple menus

    This also cannot be inherited by a widget because it need to hold
    what the menu should look like
    """

    def __init__(self, parents: list[tk.Widget], menu_builder: Callable[[], tk.Menu]):

        self.parents = parents
        self.menu_builder = menu_builder

        for parent in parents:
            parent.bind("<3>", lambda evt: parent.after(1, lambda: self.make_context_menu(evt)))

    def make_context_menu(self, evt):
        self.ctx_menu = self.menu_builder()
        self.ctx_menu.post(evt.x_root, evt.y_root)
        self.ctx_menu.bind("<1>", lambda _: self.click_tracker.unbind(), "+")
        self.ctx_menu.bind("<2>", lambda _: self.click_tracker.unbind(), "+")
        self.ctx_menu.bind("<3>", lambda _: self.click_tracker.unbind(), "+")

        self.click_tracker = TrackOutsideClick(self.parents)
        self.click_tracker.bind(self.destroy_context_menu)

    def destroy_context_menu(self):
        self.ctx_menu.destroy()
        self.click_tracker.unbind()
