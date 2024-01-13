-- Upgrade from schema_v1.1.db to schema_v1.2.db

CREATE TABLE IF NOT EXISTS milestones(
	Id	 INTEGER PRIMARY KEY,
	DueDate	 TEXT,
	Name	 TEXT,
	Category TEXT,
	Finished BOOLEAN
);

ALTER TABLE tasks RENAME TO tasks_old;
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

INSERT INTO tasks(Category, Finished, Name, Budget, Time, Used, NextAction, DueDate, Notes, DateAdded, TaskID) SELECT Category, Finished, Name, Budget, Time, Used, NextAction, DueDate, Notes, DateAdded, TaskID FROM tasks_old;
DROP TABLE tasks_old;

ALTER TABLE hyperlinks RENAME TO hyperlinks_old;
CREATE TABLE IF hyperlinks(
	Url	TEXT,
	Display TEXT,
	Task	INTEGER,
	FOREIGN KEY (Task) REFERENCES tasks(TaskID) ON DELETE CASCADE
);
INSERT INTO hyperlinks SELECT * FROM hyperlinks_old;
DROP TABLE hyperlinks_old;
