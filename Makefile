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

client1:
	@echo "Run Client 1"
	@cargo run --bin client client1.toml

client2:
	@echo "Run Client 2"
	@cargo run --bin client client2.toml

config:
	@echo "Run configurator"
	@cargo run --bin config node node.toml
	@cargo run --bin config client client.toml

node:
	@echo "Run Server"
	@cargo run --bin node node.toml

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

