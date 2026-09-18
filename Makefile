# Use these targets rather than raw cargo lines: the two that matter have a
# detail in them (the elements are embedded at build time; the package needs a
# release binary first) that a raw command gets wrong once and then nobody
# remembers why the panel did not change.

CARGO ?= cargo
PORT ?= 8082
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
ARCH ?= amd64

.PHONY: build test check proto check-proto check-schema check-text run dev schema package clean help

help:
	@sed -n 's/^\([a-z-]*\):.*## \(.*\)/  \1|\2/p' $(MAKEFILE_LIST) | column -t -s '|'

build: ## Release binary, with the console panels embedded in it
	$(CARGO) build --release

test: ## The zone compiler's tests
	$(CARGO) test --release

check: ## Build, clippy, tests, the proto, the committed schema, and that source is text
	$(CARGO) build --release
	$(CARGO) clippy --release --all-targets -- -D warnings
	$(CARGO) test --release
	@$(MAKE) --no-print-directory check-proto
	@$(MAKE) --no-print-directory check-schema
	@$(MAKE) --no-print-directory check-text

# The generated code is committed, so `cargo build` needs no protoc and a
# reviewer sees an interface change as a diff. tools/protogen is outside the
# workspace for the same reason: its generators are not dependencies of the
# daemon.
proto: ## Regenerate daemon/src/wire/ from proto/
	$(CARGO) run --quiet --manifest-path tools/protogen/Cargo.toml

# proto/ is the interface (contracts/DAEMON_LAYOUT.md), and an interface
# nothing checks is a wish. Two things are checked: protoc catches a file that
# does not parse, and the regenerated output catches committed code that no
# longer matches the schema.
#
# There was a third — a route checker, holding a hand-maintained router to the
# rpcs by reading source code. Under gRPC the generated service trait has a
# method per rpc and the compiler refuses an incomplete implementation, so
# `cargo build` is the check.
check-proto: ## Fail if the proto does not compile or the generated code is stale
	@protoc --proto_path=proto --descriptor_set_out=/dev/null \
	  proto/mousewheeld/v1/*.proto
	@$(MAKE) --no-print-directory proto
	@# Only the generated files: `mod.rs` beside them is hand-written, and a
	@# check that flagged it would fail every time somebody wrote a comment.
	@git diff --quiet -- daemon/src/wire/mousewheeld.v1.rs daemon/src/wire/service \
	  daemon/src/wire/descriptor_for_reflection.bin || { \
	  echo "daemon/src/wire/ is not what proto/ produces — the interface changed:"; \
	  git --no-pager diff --stat -- daemon/src/wire/mousewheeld.v1.rs daemon/src/wire/service \
	    daemon/src/wire/descriptor_for_reflection.bin; \
	  echo "run 'make proto' and commit the result with the change that caused it."; \
	  exit 1; \
	}

# A NUL byte in a source file makes git call it binary, and a binary file has no
# diff — so it is reviewed by nobody, silently, for as long as it takes somebody
# to notice. That happened here: three panels spent a week that way after a
# generation script interpreted a unicode escape instead of writing it.
check-text: ## Fail if any tracked source file contains a NUL byte
	@git grep -lIP '\x00' -- '*.rs' '*.js' '*.toml' '*.json' '*.md' '*.html' > /dev/null 2>&1 && { \
	  echo "a tracked source file contains a NUL byte — git will treat it as binary:"; \
	  git grep -lIP '\x00' -- '*.rs' '*.js' '*.toml' '*.json' '*.md' '*.html'; \
	  exit 1; \
	} || true

run: build ## Serve against a real board, from the installed rig config
	./target/release/mousewheeld serve --port $(PORT)

# `cargo run` without --release, deliberately: rust-embed serves the elements
# from disk in a debug build, so editing client/web/elements/*.js and reloading the
# page is enough. A release build embeds them, and a panel that did not change
# after an edit is almost always this.
dev: ## A wheel on a thread, elements served from disk, panels at http://127.0.0.1:$(PORT)/
	$(CARGO) run -- serve --simulate --port $(PORT) \
		--rig-config packaging/mousewheeld-rig-config.toml \
		--storage-dir ./dev/store

# **The zone set's schema is committed, and so is the interface.**
#
# Both belong in the repository and they are different artifacts. A zone set is
# a *file* — hand-edited, copied between rigs, reviewed in a pull request — so
# its schema must be reachable by an editor and by CI without a daemon running.
# The interface is proto/, authored rather than generated, and the daemon serves
# it as itself at /api/proto.
#
# What is gone is the generated OpenAPI document. It described the API by
# restating what the handlers happened to do, which is exactly the second
# description contracts/DAEMON_LAYOUT.md exists to prevent now that the first
# one is written by hand.
schema: build ## Regenerate docs/reference/zone-set.schema.json from the types
	@./target/release/mousewheeld schema > docs/reference/zone-set.schema.json
	@echo "docs/reference/zone-set.schema.json"

check-schema: ## Fail if the committed schema is not what the code produces
	@$(MAKE) --no-print-directory schema
	@git diff --quiet -- docs/reference/zone-set.schema.json || { \
	  echo "docs/reference/zone-set.schema.json is out of date — the type changed:"; \
	  git --no-pager diff -- docs/reference/zone-set.schema.json | head -40; \
	  echo "commit the regenerated schema with the change that caused it."; \
	  exit 1; \
	}

package: build ## deb and rpm, from packaging/nfpm.yaml
	@mkdir -p dist
	VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f packaging/nfpm.yaml -p deb -t dist/
	VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f packaging/nfpm.yaml -p rpm -t dist/

clean:
	$(CARGO) clean
	rm -rf dist
