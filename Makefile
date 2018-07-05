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

run:
	@echo "Build..."
	@cargo run

build:
	@echo Build debug version...
	@RUSTFLAGS="-D warnings" cargo build
	@echo Done.

release:
	@echo Build release version...
	@cargo build --release
	@echo Done.

fmt:
	@cargo fmt

clippy:
	@cargo clippy

kcov:
	@docker run --rm --security-opt seccomp=unconfined -v $(pwd):/app -w /app ragnaroek/kcov:v33
