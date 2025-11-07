.PHONY: all build test test-single clean

all: build test

build:
	@echo "Building orchestrator binary..."
	mkdir -p bin
	cd orchestrator && go build -o ./bin/orchestrator
	@echo "Building go-handler binary..."
	cd go-handler/go-bitcoinkernel && $(MAKE) build-kernel && $(MAKE) build
	cd go-handler && go build -o ./bin/go-handler
	@echo "Building rust-handler binary..."
	cd rust-handler && cargo build --release
	@echo "Build complete!"

test:
	@echo "Running all conformance tests with go-handler..."
	-./orchestrator/bin/orchestrator -handler ./go-handler/bin/go-handler -testdir testdata
	@echo "Running all conformance tests with rust-handler..."
	-./orchestrator/bin/orchestrator -handler ./rust-handler/target/release/rust-handler -testdir testdata

test-single:
	@if [ -z "$(TEST)" ]; then \
		echo "Error: TEST variable not set. Usage: make test-single TEST=testdata/chainstate_basic.json"; \
		exit 1; \
	fi
	@echo "Running test with go-handler: $(TEST)"
	./bin/orchestrator -handler ./go-handle/bin/go-handler -testfile $(TEST)
	@echo "Running test with rust-handler: $(TEST)"
	./bin/orchestrator -handler ./rust-handler/target/release/rust-handler -testfile $(TEST)

clean:
	@echo "Cleaning build artifacts..."
	cd go-handler/go-bitcoinkernel && $(MAKE) clean
	cd orchestrator && go clean && rm -rf build
	cd go-handler && go clean && rm -rf build
	cd rust-handler && cargo clean