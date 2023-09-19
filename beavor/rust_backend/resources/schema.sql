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
