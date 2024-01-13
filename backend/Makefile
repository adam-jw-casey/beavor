VERSION = v1.1
RESOURCES = resources
UPGRADE = $(RESOURCES)/upgrade
TARGET = $(RESOURCES)/beavor_$(VERSION).db

ifneq ($(wildcard $(RESOURCES)/*.db),)
	ifneq ($(wildcard $(TARGET)),)
		MODE = none
	else
		MODE = upgrade
	endif
else
	MODE = new
endif

default: .env

.env: $(MODE) $(RESOURCES)/schema.sql
	@echo "DATABASE_URL=sqlite:$(abspath $(TARGET))" > $@

none:

new: $(RESOURCES)/schema.sql
	sqlite3 $(TARGET) < $<

upgrade: $(TARGET)

$(RESOURCES)/beavor_v1.1.db: $(RESOURCES)/schema.db $(UPGRADE)/upgrade_v1.1.sql
	sqlite3 $< < $(UPGRADE)/upgrade_v1.1.sql
	mv $< $@

$(RESOURCES)/beavor_v1.2.db: $(RESOURCES)/schema.db $(UPGRADE)/upgrade_v1.2.sql
	sqlite3 $< < $(UPGRADE)/upgrade_v1.2.sql
	mv $< $@
