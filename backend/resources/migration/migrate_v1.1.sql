-- Upgrade from schema.db to schema_v1.1.db
ALTER TABLE tasks RENAME TO tasks_old;
CREATE TABLE tasks(
	Category   TEXT,
	Finished   BOOLEAN,
	Name       TEXT,
	Budget     INTEGER,
	Time       INTEGER,
	Used       INTEGER,
	NextAction TEXT,
	DueDate    TEXT,
	Notes      TEXT,
	DateAdded  TEXT,
	TaskID	   INTEGER PRIMARY KEY
);

INSERT INTO tasks(Category, Finished, Name, Budget, Time, Used, NextAction, DueDate, Notes, DateAdded) SELECT Category, Finished, Name, Budget, Time, Used, NextAction, DueDate, Notes, DateAdded FROM tasks_old;
DROP TABLE tasks_old;

CREATE TABLE hyperlinks(
	Url	TEXT,
	Display TEXT,
	Task	INTEGER,
	FOREIGN KEY (Task) REFERENCES tasks(TaskID) ON DELETE CASCADE
);
