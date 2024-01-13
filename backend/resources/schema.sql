CREATE TABLE IF NOT EXISTS tasks(
	Category   TEXT,
	Finished   BOOLEAN,
	Name       TEXT,
	Budget     INTEGER,
	Time       INTEGER,
	Used       INTEGER,
	NextAction TEXT,
	DueDate    TEXT,
	StartMilestone INTEGER,
	EndMilestone   INTEGER,
	Notes      TEXT,
	DateAdded  TEXT,
	TaskID	   INTEGER PRIMARY KEY,
	FOREIGN KEY (StartMilestone) REFERENCES milestones(ID),
	FOREIGN KEY (EndMilestone) REFERENCES milestones(ID)
);

CREATE TABLE IF NOT EXISTS days_off(
	Day	TEXT UNIQUE,
	Reason	TEXT CHECK(Reason IN ('vacation', 'stat_holiday', 'travel'))
);

CREATE TABLE IF NOT EXISTS hyperlinks(
	Url	TEXT,
	Display TEXT,
	Task	INTEGER,
	FOREIGN KEY (Task) REFERENCES tasks(TaskID) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS milestones(
	Id	 INTEGER PRIMARY KEY,
	DueDate	 TEXT,
	Name	 TEXT,
	Category TEXT,
	Finished BOOLEAN
);
