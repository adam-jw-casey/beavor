default: .env

.env: resources/schema.db
	@echo "DATABASE_URL=sqlite:$(abspath $<)" > $@

resources/schema.db: resources/schema.sql
	cd $(@D); rm -f $(@F); sqlite3 $(@F) < schema.sql
