PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tasks(
	TaskID		  INTEGER NOT NULL PRIMARY KEY,
	Name		  TEXT,
	Finished      	  BOOLEAN NOT NULL CHECK (Finished IN (0,1)),
	TimeBudgeted  	  INTEGER,
	TimeNeeded    	  INTEGER,
	TimeUsed      	  INTEGER,
	Available     	  TEXT,
	DueDeliverable	  INTEGER,
	PrereqDeliverable INTEGER,
	Notes		  TEXT,
	DateAdded     	  TEXT,
	FOREIGN KEY (DueDeliverable) REFERENCES deliverables (DeliverableID),
	FOREIGN KEY (PrereqDeliverable) REFERENCES deliverables (DeliverableID)
	);

CREATE TABLE IF NOT EXISTS projects(
	ProjectID     INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT,
	Category      INTEGER,
	UNIQUE(Name, Category),
	FOREIGN KEY (Category) REFERENCES categories (CategoryID)
	);

CREATE TABLE IF NOT EXISTS deliverables(
	DeliverableID INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT,
	Project       INTEGER,
	DueDate	      TEXT,
	Notes	      TEXT,
	FOREIGN KEY (Project) REFERENCES projects (ProjectID)
	);

CREATE TABLE IF NOT EXISTS externals(
	ExternalID      INTEGER NOT NULL PRIMARY KEY,
	Name		TEXT,
	Link	      	TEXT,
	DeliverableID	INTEGER,
	TaskID		INTEGER,
	FOREIGN KEY (DeliverableID) REFERENCES deliverables (DeliverableID),
	FOREIGN KEY (DeliverableID) REFERENCES tasks (TaskID)
	);

CREATE TABLE IF NOT EXISTS categories(
	CategoryID    INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT UNIQUE
	)
