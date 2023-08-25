from tkinter import Widget, Entry, Label, Text

class SensibleReturnWidget(Widget):
    def grid(self, *args, **kwargs):
        super().grid(*args, **kwargs)
        return self

class EntrySR(Entry, SensibleReturnWidget):
    pass

class LabelSR(Label, SensibleReturnWidget):
    pass

class TextSR(Text, SensibleReturnWidget):
    pass
