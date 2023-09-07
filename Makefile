BACKEND = beavor/rust_backend
ifeq ($(OS),Windows_NT)
	LIB_TARGET = beavor/backend.pyd
	LIB_COMPILED = backend.dll
else
	LIB_TARGET = beavor/backend.so
	LIB_COMPILED = libbackend.so
endif

default: debug

release: $(BACKEND)/target/release/$(LIB_COMPILED)
	cp $< $(LIB_TARGET)

debug: $(BACKEND)/target/debug/$(LIB_COMPILED)
	cp $< $(LIB_TARGET)

test:
	cd $(BACKEND)/; cargo test

$(BACKEND)/target/release/$(LIB_COMPILED): $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/schema.db
	cd $(BACKEND)/; cargo build --release
	touch $@

$(BACKEND)/target/debug/$(LIB_COMPILED): $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/schema.db
	cd $(BACKEND)/; cargo build
	touch $@

$(BACKEND)/resources/schema.db: $(BACKEND)/resources/schema.sql
	cd $(@D); rm -f $(@F); sqlite3 $(@F) < schema.sql

clean:
	@rm __pycache__ -rf
	@rm beavor/__pycache__ -rf
	@rm $(LIB_TARGET)
	@cd $(BACKEND); cargo clean
