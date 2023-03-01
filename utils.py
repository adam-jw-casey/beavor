import datetime
import numpy as np

#Like .ljust, but truncates to length if necessary
def ljusttrunc(text: str, length: int) -> str:
  return text[:length].ljust(length)

def greenRedScale(low: float, high: float, val: float) -> str:
  #linear interpolation bounded on [0,1]
  frac = max(0, min(1, (val - low) / (high - low)))
  if frac > 0.5:
    red = 255
    green = int((2-2*frac) * 255)
  else:
    red = int((2*frac) * 255)
    green = 255

  return "#{}{}00".format(str(hex(red)[2:]).rjust(2,'0'), str(hex(green)[2:]).rjust(2,'0'))

#Surrounds the string inner with string outer, reversed the second time, and returns the result
def surround(inner: str, outer: str) -> str:
  return outer + inner + outer[::-1]

#Double up single quotes in a string
def escapeSingleQuotes(text) -> str:
  return "".join([c if c != "'" else c+c for c in text])

# Takes a string "YYYY-MM-DD"
def daysBetween(d1: str, d2: str) -> int:
  d1d = YMDstr2date(d1)
  d2d = YMDstr2date(d2)
  return (d2d - d1d).days

# takes strings "%Y-%m-%d"
# inclusive of start and end date
def workDaysBetween(d1: str | datetime.date, d2: str) -> int:
  return int(np.busday_count(d1, (YMDstr2date(d2) + datetime.timedelta(days=1))))

def YMDstr2date(dateString: str) -> datetime.date:
  return datetime.datetime.strptime(dateString, "%Y-%m-%d").date()

def date2YMDstr(dateVar: datetime.date) -> str:
  return dateVar.strftime("%Y-%m-%d")

def todayStr() -> str:
  return date2YMDstr(todayDate())

def todayDate() -> datetime.date:
  return datetime.date.today()
