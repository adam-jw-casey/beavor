BACKEND = beavor/rust_backend
ifeq ($(OS),Windows_NT)
	LIB_TARGET = beavor/backend.pyd
else
	LIB_TARGET = beavor/backend.so
endif


default: debug

release: $(BACKEND)/target/release/libbackend.so
	cp $< $(LIB_TARGET)

debug: $(BACKEND)/target/debug/libbackend.so
	cp $< $(LIB_TARGET)

$(BACKEND)/target/release/libbackend.so: $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/schema.db
	cd $(BACKEND)/; cargo build --release
	touch $@

$(BACKEND)/target/debug/libbackend.so: $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/schema.db
	cd $(BACKEND)/; cargo build
	touch $@

$(BACKEND)/resources/schema.db: $(BACKEND)/resources/schema.sql
	cd $(@D); rm -f $(@F); sqlite3 $(@F) < schema.sql

clean:
	@rm __pycache__ -rf
	@rm beavor/__pycache__ -rf
	@rm beavor/backend.so
	@cd $(BACKEND); cargo clean
