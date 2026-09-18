# Use these targets rather than raw cargo lines: the two that matter have a
# detail in them (the elements are embedded at build time; the package needs a
# release binary first) that a raw command gets wrong once and then nobody
# remembers why the panel did not change.

CARGO ?= cargo
PORT ?= 8082
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
ARCH ?= amd64

.PHONY: build test check check-openapi run dev openapi package clean help

help:
	@sed -n 's/^\([a-z-]*\):.*## \(.*\)/  \1|\2/p' $(MAKEFILE_LIST) | column -t -s '|'

build: ## Release binary, with the console panels embedded in it
	$(CARGO) build --release

test: ## The zone compiler's tests
	$(CARGO) test --release

check: ## Build, clippy, tests, and the committed API document, as CI would
	$(CARGO) build --release
	$(CARGO) clippy --release --all-targets -- -D warnings
	$(CARGO) test --release
	@$(MAKE) --no-print-directory check-openapi

run: build ## Serve against a real board, from the installed rig config
	./target/release/mousewheeld serve --port $(PORT)

# `cargo run` without --release, deliberately: rust-embed serves the elements
# from disk in a debug build, so editing web/elements/*.js and reloading the
# page is enough. A release build embeds them, and a panel that did not change
# after an edit is almost always this.
dev: ## A wheel on a thread, elements served from disk, panels at http://127.0.0.1:$(PORT)/
	$(CARGO) run -- serve --simulate --port $(PORT) \
		--rig-config packaging/mousewheeld-rig-config.toml \
		--storage-dir ./dev/store

# The document is GENERATED from the types and COMMITTED to the repository, and
# it needs to be both. Generated, so there is no second copy of the schema to
# drift from the code. Committed, so the interface is a file a reviewer reads
# and a pull request diffs, instead of something that only exists once a binary
# is running. `check` fails when the two disagree.
openapi: build ## Regenerate docs/reference/openapi.json from the types
	@./target/release/mousewheeld serve --simulate --port 8099 --storage-dir ./dev/store & \
	 pid=$$!; sleep 1; \
	 curl -fsS http://127.0.0.1:8099/api/openapi.json \
	   | python3 -m json.tool --indent 2 > docs/reference/openapi.json; \
	 kill $$pid; \
	 echo "docs/reference/openapi.json"

check-openapi: ## Fail if the committed document is not what the code produces
	@$(MAKE) --no-print-directory openapi
	@git diff --quiet -- docs/reference/openapi.json || { \
	  echo "docs/reference/openapi.json is out of date — the API changed:"; \
	  git --no-pager diff --stat -- docs/reference/openapi.json; \
	  echo "commit the regenerated document with the change that caused it."; \
	  exit 1; \
	}

package: build ## deb and rpm, from packaging/nfpm.yaml
	@mkdir -p dist
	VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f packaging/nfpm.yaml -p deb -t dist/
	VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f packaging/nfpm.yaml -p rpm -t dist/

clean:
	$(CARGO) clean
	rm -rf dist
