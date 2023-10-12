--TODO - Add a table to log times worked. Could be used to "undo" a timer that overran, as well as allowing analysis of time worked
CREATE TABLE IF NOT EXISTS tasks(
	Category   TEXT,
	Finished   BOOLEAN,
	Name       TEXT,
	Budget     INTEGER,
	Time       INTEGER,
	Used       INTEGER,
	NextAction TEXT,
	DueDate    TEXT,
	Notes      TEXT,
	DateAdded  TEXT
);

CREATE TABLE IF NOT EXISTS days_off(
	Day	TEXT UNIQUE,
	Reason	TEXT CHECK(Reason IN ('vacation', 'stat_holiday', 'travel'))
);
