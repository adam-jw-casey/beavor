from tkinter import Widget, Entry, Label, Text, Frame, Button, Checkbutton, Canvas, Scrollbar

class SensibleReturnWidget(Widget):
    def grid(self, *args, **kwargs):
        super().grid(*args, **kwargs)
        return self

    def pack(self, *args, **kwargs):
        super().pack(*args, **kwargs)
        return self

    def bind(self, *args, **kwargs):
        super().bind(*args, **kwargs)
        return self

class EntrySR(Entry, SensibleReturnWidget):
    pass

class LabelSR(Label, SensibleReturnWidget):
    pass

class TextSR(Text, SensibleReturnWidget):
    pass

class FrameSR(Frame, SensibleReturnWidget):
    pass

class ButtonSR(Button, SensibleReturnWidget):
    pass

class CheckbuttonSR(Checkbutton, SensibleReturnWidget):
    pass

class CanvasSR(Canvas, SensibleReturnWidget):
    pass

class ScrollbarSR(Scrollbar, SensibleReturnWidget):
    pass
