BACKEND = beavor/rust_backend

default: debug

release: $(BACKEND)/target/release/libbackend.so
	cp $(BACKEND)/target/release/libbackend.so beavor/backend.so

debug: $(BACKEND)/target/debug/libbackend.so
	cp $(BACKEND)/target/debug/libbackend.so beavor/backend.so

$(BACKEND)/target/release/libbackend.so: $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/schema.db
	cd $(BACKEND)/; cargo build --release

$(BACKEND)/target/debug/libbackend.so: $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/schema.db
	cd $(BACKEND)/; cargo build

$(BACKEND)/resources/schema.db: $(BACKEND)/resources/schema.sql
	cd $(BACKEND)/resources; sqlite3 schema.db < schema.sql

clean:
	@rm __pycache__ -rf
	@rm beavor/__pycache__ -rf
	@rm beavor/backend.so
	@cd $(BACKEND); cargo clean
