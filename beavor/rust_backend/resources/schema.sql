--TODO this should be renamed to tasks
CREATE TABLE IF NOT EXISTS worklist(
	Category   TEXT,
	O          TEXT CHECK (O in ('O', 'X')),
	Task       TEXT,
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
