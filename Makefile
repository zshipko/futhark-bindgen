PREFIX?=$(HOME)/.local

build:
	cargo build

test: test-rust test-ocaml

test-rust:
	cd examples/rust && cargo test
	
test-ocaml:
	cd examples/ocaml && dune clean && dune runtest
	
install:
	cargo build --release
	install -D target/release/futhark-bindgen $(PREFIX)/bin/futhark-bindgen
	
uninstall:
	rm -f $(PREFIX)/bin/futhark-bindgen
