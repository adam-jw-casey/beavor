default: debug

release: beavor/rust_backend/target/release/libbackend.so
	cp beavor/rust_backend/target/debug/libbackend.so beavor/backend.so

debug: beavor/rust_backend/target/debug/libbackend.so
	cp beavor/rust_backend/target/debug/libbackend.so beavor/backend.so

beavor/rust_backend/target/release/libbackend.so: beavor/rust_backend/Cargo.toml beavor/rust_backend/src/* beavor/rust_backend/.env
	cd beavor/rust_backend/; cargo build --release

beavor/rust_backend/target/debug/libbackend.so: beavor/rust_backend/Cargo.toml beavor/rust_backend/src/* beavor/rust_backend/.env
	cd beavor/rust_backend/; cargo build

clean:
	@rm __pycache__ -rf
	@rm beavor/__pycache__ -rf
	@rm beavor/backend.so
	@cd beavor/rust_backend; cargo clean
