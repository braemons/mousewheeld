# Use these targets rather than raw uv lines: the generated stubs are committed
# and there is a check that holds them to proto/, which a raw command skips.

UV ?= uv
PROTO_ROOT := ../../proto
PROTO_OUT  := mousewheeld/_proto
PROTOS     := $(wildcard $(PROTO_ROOT)/mousewheeld/v1/*.proto)

.PHONY: help proto generate-into check-proto test test-unit typecheck check build clean

help:
	@sed -n 's/^\([a-z-]*\):.*## \(.*\)/  \1|\2/p' $(MAKEFILE_LIST) | column -t -s '|'

# The generated code is committed (contracts/DAEMON_LAYOUT.md), so an install
# needs no protoc and grpcio-tools is a dev dependency rather than a runtime
# one. A reviewer sees an interface change as a diff.
proto: ## Regenerate mousewheeld/_proto/ from ../../proto
	@$(MAKE) --no-print-directory generate-into OUT=$(PROTO_OUT)
	@echo "$(PROTO_OUT)/"

generate-into:
	@mkdir -p $(OUT)
	@$(UV) run --group dev python -m grpc_tools.protoc \
	  --proto_path=$(PROTO_ROOT) \
	  --python_out=$(OUT) --pyi_out=$(OUT) --grpc_python_out=$(OUT) \
	  $(PROTOS)

# Against a fresh generation rather than against git: a *new* generated file is
# untracked, and `git diff` says nothing at all about an untracked file. A
# staleness check that goes quiet the first time a message is added is worse
# than no check — which is how this one was found.
check-proto: ## Fail if the committed stubs are not what proto/ produces
	@rm -rf build/proto-check
	@$(MAKE) --no-print-directory generate-into OUT=build/proto-check
	@# --exclude=__pycache__: a bytecode cache is not the interface.
	@diff -r --exclude=__pycache__ build/proto-check $(PROTO_OUT) || { \
	  echo "$(PROTO_OUT)/ is not what proto/ produces — the interface changed."; \
	  echo "run 'make proto' and commit the result with the change that caused it."; \
	  exit 1; \
	}

# The suite needs a daemon, and starts one itself: `--simulate`, on a free
# port, in a temporary directory. It skips rather than fails when there is no
# release binary, so a checkout with no Rust toolchain still runs the unit
# tests — but on this repository, `cargo build --release` first.
test: ## Everything: the seam with no daemon, and the client against one
	@$(UV) run --group dev pytest -q

test-unit: ## Only the seam — no daemon, no network
	@$(UV) run --group dev pytest -q tests/test_convert.py

typecheck: ## ty
	@$(UV) run --group dev ty check mousewheeld tests

check: check-proto typecheck test ## The proto, the types, and both suites

build: proto ## A wheel
	@$(UV) build

clean:
	@rm -rf dist build .pytest_cache .venv
