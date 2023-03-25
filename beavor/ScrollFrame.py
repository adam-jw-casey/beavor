import tkinter as tk
import platform

class ScrollFrame(tk.LabelFrame):
    def __init__(self, parent: tk.Frame | tk.LabelFrame, text: str):
        super().__init__(parent, text=text) # create a frame (self)

        self.canvas = tk.Canvas(self, borderwidth=0) #place canvas on self
        self.viewPort = tk.Frame(self.canvas) #place a frame on the canvas, this frame will hold the child widgets
        self.vsb = tk.Scrollbar(self, orient="vertical", command=self.canvas.yview) #place a scrollbar on self
        self.canvas.configure(yscrollcommand=self.vsb.set) #attach scrollbar action to scroll of canvas

        self.vsb.pack(side="right", fill="y") #pack scrollbar to right of self
        self.canvas.pack(side="left", fill="both", expand=True) #pack canvas to left of self and expand to fill
        self.canvas_window = self.canvas.create_window((4,4), window=self.viewPort, anchor="nw", tags="self.viewPort")

        self.viewPort.bind("<Configure>", self.onFrameConfigure) #bind an event whenever the size of the viewPort frame changes.
        self.viewPort.grid_columnconfigure(0, weight=1)

        self.canvas.bind("<Configure>", self.onCanvasConfigure) #bind an event whenever the size of the canvas frame changes.

        self.bind('<Enter>', self.onEnter) # bind wheel events when the cursor enters the control
        self.bind('<Leave>', self.onLeave) # unbind wheel events when the cursor leaves the control

        self.onFrameConfigure(None) #perform an initial stretch on render, otherwise the scroll region has a tiny border until the first resize

    def onFrameConfigure(self, _):
        '''Reset the scroll region to encompass the inner frame'''
        self.canvas.configure(scrollregion=self.canvas.bbox("all")) #whenever the size of the frame changes, alter the scroll region respectively.

    def onCanvasConfigure(self, event: tk.Event):
        '''Reset the canvas window to encompass inner frame when required'''
        canvas_width = event.width
        self.canvas.itemconfig(self.canvas_window, width = canvas_width) #whenever the size of the canvas changes alter the window region respectively.

    def onMouseWheel(self, event: tk.Event):  # cross platform scroll wheel event
        canvas_height = self.canvas.winfo_height()
        rows_height = self.canvas.bbox("all")[3]

        if rows_height > canvas_height:
            if platform.system() == 'Windows':
                self.canvas.yview_scroll(int(-1* (event.delta/120)), "units")
            elif platform.system() == 'Darwin':
                self.canvas.yview_scroll(int(-1 * event.delta), "units")
            else:
                if event.num == 4:
                    self.canvas.yview_scroll( -1, "units" )
                elif event.num == 5:
                    self.canvas.yview_scroll( 1, "units" )

    def onEnter(self, _): # bind wheel events when the cursor enters the control
        if platform.system() == 'Linux':
            self.canvas.bind_all("<Button-4>", self.onMouseWheel)
            self.canvas.bind_all("<Button-5>", self.onMouseWheel)
        else:
            self.canvas.bind_all("<MouseWheel>", self.onMouseWheel)

    def onLeave(self, _): # unbind wheel events when the cursor leaves the control
        if platform.system() == 'Linux':
            self.canvas.unbind_all("<Button-4>")
            self.canvas.unbind_all("<Button-5>")
        else:
            self.canvas.unbind_all("<MouseWheel>")
