PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tasks(
	id		  INTEGER NOT NULL PRIMARY KEY,
	Name		  TEXT 	  NOT NULL,
	Status      	  TEXT	  NOT NULL,
	TimeBudgeted  	  INTEGER NOT NULL,
	TimeNeeded    	  INTEGER NOT NULL,
	TimeUsed      	  INTEGER NOT NULL,
	Available     	  TEXT    NOT NULL,
	DueDeliverable	  INTEGER NOT NULL,
	PrereqDeliverable INTEGER,
	Notes		  TEXT    NOT NULL,
	DateAdded     	  TEXT    NOT NULL,
	FOREIGN KEY (DueDeliverable)    REFERENCES deliverables (id) ON DELETE CASCADE,
	FOREIGN KEY (PrereqDeliverable) REFERENCES deliverables (id)
	);

CREATE TABLE IF NOT EXISTS projects(
	id     	      INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT    NOT NULL,
	Category      INTEGER NOT NULL,
	UNIQUE(Name, Category),
	FOREIGN KEY (Category) REFERENCES categories (id) ON DELETE CASCADE
	);

CREATE TABLE IF NOT EXISTS deliverables(
	id	      INTEGER NOT NULL PRIMARY KEY,
	Name	      TEXT    NOT NULL,
	Project       INTEGER NOT NULL,
	DueDate	      TEXT    NOT NULL,
	Finished      BOOLEAN NOT NULL CHECK (Finished IN (0,1)),
	Notes	      TEXT    NOT NULL,
	FOREIGN KEY (Project) REFERENCES projects (id) ON DELETE CASCADE
	);

CREATE TABLE IF NOT EXISTS externals(
	id	        INTEGER NOT NULL PRIMARY KEY,
	Name		TEXT    NOT NULL,
	Link	      	TEXT    NOT NULL,
	Deliverable	INTEGER NOT NULL,
	FOREIGN KEY (Deliverable) REFERENCES deliverables (id) ON DELETE CASCADE
	);

CREATE TABLE IF NOT EXISTS categories(
	id	      INTEGER     NOT NULL PRIMARY KEY,
	Name	      TEXT UNIQUE NOT NULL
	)
