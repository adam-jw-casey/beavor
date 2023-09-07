beavor/backend.so: beavor/rust_backend/target/debug/libbackend.so
	cp beavor/rust_backend/target/debug/libbackend.so beavor/backend.so

beavor/rust_backend/target/debug/libbackend.so: beavor/rust_backend/Cargo.toml beavor/rust_backend/src/*
	cd beavor/rust_backend/; cargo build

clean:
	@rm __pycache__ -rf
	@rm beavor/__pycache__ -rf
	@rm beavor/backend.so
	@cd beavor/rust_backend; cargo clean
