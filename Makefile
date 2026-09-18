# Use these targets rather than raw cargo lines: the two that matter have a
# detail in them (the elements are embedded at build time; the package needs a
# release binary first) that a raw command gets wrong once and then nobody
# remembers why the panel did not change.

CARGO ?= cargo
PORT ?= 8082
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
ARCH ?= amd64

.PHONY: build test check check-proto check-schema check-text run dev schema openapi package clean help

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

# proto/ is the interface (contracts/DAEMON_LAYOUT.md), and an interface
# nothing checks is a wish. Two things are checked and they catch different
# mistakes: protoc catches a file that does not parse, and check_routes.py
# catches a handler wired into the router with no rpc above it — a piece of
# public API that exists and is written down nowhere.
check-proto: ## Fail if the proto does not compile, or does not match the router
	@protoc --proto_path=proto --descriptor_set_out=/dev/null \
	  proto/mousewheeld/v1/*.proto proto/braemons/v1/route.proto
	@python3 tools/check_routes.py

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

# **The zone set's schema is committed; the API document is not.**
#
# They are different kinds of artifact. A zone set is a *file* — hand-edited,
# copied between rigs, reviewed in a pull request — so its schema belongs in the
# repository where an editor and CI can reach it without a daemon running, and a
# change to it should show up in a diff. The API document is 2000 lines of
# generated JSON that nobody reads; it is served at `/api/openapi.json` for
# clients and code generators, and what a person reads instead is
# docs/reference/api.md, written by hand.
schema: build ## Regenerate docs/reference/zone-set.schema.json from the types
	@./target/release/mousewheeld serve --simulate --port 8099 --storage-dir ./dev/store & \
	 pid=$$!; sleep 1; \
	 curl -fsS http://127.0.0.1:8099/api/zone-sets/schema \
	   | python3 -m json.tool --indent 2 > docs/reference/zone-set.schema.json; \
	 kill $$pid; \
	 echo "docs/reference/zone-set.schema.json"

check-schema: ## Fail if the committed schema is not what the code produces
	@$(MAKE) --no-print-directory schema
	@git diff --quiet -- docs/reference/zone-set.schema.json || { \
	  echo "docs/reference/zone-set.schema.json is out of date — the type changed:"; \
	  git --no-pager diff -- docs/reference/zone-set.schema.json | head -40; \
	  echo "commit the regenerated schema with the change that caused it."; \
	  exit 1; \
	}

openapi: build ## Write the served API document to dist/, for a code generator
	@mkdir -p dist
	@./target/release/mousewheeld serve --simulate --port 8099 --storage-dir ./dev/store & \
	 pid=$$!; sleep 1; \
	 curl -fsS http://127.0.0.1:8099/api/openapi.json -o dist/openapi.json; \
	 kill $$pid; \
	 echo "dist/openapi.json"

package: build ## deb and rpm, from packaging/nfpm.yaml
	@mkdir -p dist
	VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f packaging/nfpm.yaml -p deb -t dist/
	VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f packaging/nfpm.yaml -p rpm -t dist/

clean:
	$(CARGO) clean
	rm -rf dist
