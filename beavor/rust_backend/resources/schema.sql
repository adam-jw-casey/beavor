PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tasks(
	TaskID		  INTEGER NOT NULL PRIMARY KEY,
	Name		  TEXT 	  NOT NULL,
	Finished      	  BOOLEAN NOT NULL CHECK (Finished IN (0,1)),
	TimeBudgeted  	  INTEGER NOT NULL,
	TimeNeeded    	  INTEGER NOT NULL,
	TimeUsed      	  INTEGER NOT NULL,
	Available     	  TEXT    NOT NULL,
	DueDeliverable	  INTEGER NOT NULL,
	PrereqDeliverable INTEGER NOT NULL,
	Notes		  TEXT    NOT NULL,
	DateAdded     	  TEXT    NOT NULL,
	FOREIGN KEY (DueDeliverable)    REFERENCES deliverables (DeliverableID),
	FOREIGN KEY (PrereqDeliverable) REFERENCES deliverables (DeliverableID)
	);

CREATE TABLE IF NOT EXISTS projects(
	ProjectID     INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT    NOT NULL,
	Category      INTEGER NOT NULL,
	UNIQUE(Name, Category),
	FOREIGN KEY (Category) REFERENCES categories (CategoryID)
	);

CREATE TABLE IF NOT EXISTS deliverables(
	DeliverableID INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT    NOT NULL,
	Project       INTEGER NOT NULL,
	DueDate	      TEXT    NOT NULL,
	Finished      BOOLEAN NOT NULL CHECK (Finished IN (0,1)),
	Notes	      TEXT    NOT NULL,
	FOREIGN KEY (Project) REFERENCES projects (ProjectID)
	);

CREATE TABLE IF NOT EXISTS externals(
	ExternalID      INTEGER NOT NULL PRIMARY KEY,
	Name		TEXT    NOT NULL,
	Link	      	TEXT    NOT NULL,
	DeliverableID	INTEGER NOT NULL,
	FOREIGN KEY (DeliverableID) REFERENCES deliverables (DeliverableID)
	);

CREATE TABLE IF NOT EXISTS categories(
	CategoryID    INTEGER     NOT NULL PRIMARY KEY,
	Name	      TEXT UNIQUE NOT NULL
	)
