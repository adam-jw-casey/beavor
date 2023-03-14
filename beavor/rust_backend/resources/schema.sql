PRAGMA foreign_keys = ON; -- TODO this needs to be executed on every connection

CREATE TABLE IF NOT EXISTS tasks(
	Name		  TEXT,
	Finished      	  BOOLEAN NOT NULL CHECK (Finished IN (0,1)),
	TimeBudgeted  	  INTEGER,
	TimeNeeded    	  INTEGER,
	TimeUsed      	  INTEGER,
	Available     	  TEXT, -- Either a date, Any, or Deliverable
	PrereqDeliverable INTEGER,
	Notes		  TEXT,
	DateAdded     	  TEXT,
	FOREIGN KEY (PrereqDeliverable) REFERENCES deliverables (rowid)
	);

CREATE TABLE IF NOT EXISTS projects(
	Name	      TEXT,
	Category      INTEGER,
	UNIQUE(Name, Category),
	FOREIGN KEY (Category) REFERENCES categories (rowid)
	);

CREATE TABLE IF NOT EXISTS deliverables(
	Name	      TEXT,
	Project       INTEGER,
	DueDate	      TEXT, --an ISO date, None, or ASAP
	Notes	      TEXT,
	FOREIGN KEY (Project) REFERENCES projects (rowid)
	);

CREATE TABLE IF NOT EXISTS externals(
	Name		TEXT,
	Link	      	TEXT,
	DeliverableID	INTEGER,
	TaskID		INTEGER,
	FOREIGN KEY (DeliverableID) REFERENCES deliverables (rowid),
	FOREIGN KEY (DeliverableID) REFERENCES tasks (rowid)
	);

CREATE TABLE IF NOT EXISTS categories(
	Name	      TEXT UNIQUE
	)
