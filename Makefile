#
# Makefile
# @author Evgeny Ukhanov <mrlsd@ya.ru>
#

.PHONY: check, run, build, release, test
default: check

test:
	@echo Run tests...
	@cargo test -- --nocapture
	@echo Done.

check:
	@cargo check

client:
	@echo "Run Client"
	@cargo run --bin client

config:
	@echo "Run configurator"
	@cargo run --bin config node node.toml
	@cargo run --bin config client client.toml

server:
	@echo "Run Server"
	@cargo run --bin server

build:
	@echo Build debug version...
	@cargo build
	@echo Done.

release:
	@echo Build release version...
	@cargo build --release
	@echo Done.

fmt:
	@cargo fmt

