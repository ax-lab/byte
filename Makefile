.PHONY: build test test-cargo test-go test-byte

all: build

test: test-cargo test-go test-byte
	@echo
	@echo ======================================================
	@echo Tests passed!!!
	@echo ======================================================
	@echo
	
test-cargo:
	@echo
	@echo ------------------------------------------------------
	@echo :: Cargo tests
	@echo ------------------------------------------------------
	@echo
	@cd byte-rs && cargo test $(cargo)
	
test-go:
	@echo
	@echo ------------------------------------------------------
	@echo :: Go tests
	@echo ------------------------------------------------------
	@echo
	@cd bootstrap && go test
	@echo

test-byte:
	@echo
	@echo ------------------------------------------------------
	@echo :: Byte tests
	@echo ------------------------------------------------------
	@go run ./byte.go test
	
build:
	@go build ./byte.go
	@cd byte-rs && cargo build $(cargo)
