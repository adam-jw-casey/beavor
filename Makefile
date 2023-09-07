BACKEND = beavor/rust_backend
LIB_TARGET = beavor/backend.so

default: debug

check:
	cd $(BACKEND); cargo check
	pyright .

release: $(BACKEND)/target/release/libbackend.so python_dependencies
	cp $< $(LIB_TARGET)
	@echo "Successully built release library"

debug: $(BACKEND)/target/debug/libbackend.so python_dependencies
	cp $< $(LIB_TARGET)
	@echo "Successfully built debug library"

$(BACKEND)/target/release/libbackend.so: $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/dummy.db
	cd $(BACKEND)/; cargo build --release
	touch $@

$(BACKEND)/target/debug/libbackend.so: $(BACKEND)/Cargo.toml $(BACKEND)/src/* $(BACKEND)/.env $(BACKEND)/resources/dummy.db
	cd $(BACKEND)/; cargo build
	touch $@

$(BACKEND)/resources/dummy.db: $(BACKEND)/resources/schema.sql
	cd $(@D); rm -f $(@F); sqlite3 $(@F) < schema.sql

clean:
	@rm __pycache__ -rf
	@rm beavor/__pycache__ -rf
	@rm beavor/backend.so
	@cd $(BACKEND); cargo clean

python_dependencies: .pydepsupdated
	@:

.pydepsupdated: requirements.txt .pipreqsinstalled
	pip3 install -r $<
	@touch $@

requirements.txt: launch beavor/*.py
	pipreqs . --force

.pipreqsinstalled:
	pip3 install pipreqs
	@touch $@
