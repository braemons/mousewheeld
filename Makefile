# Use these targets rather than raw cargo lines: the two that matter have a
# detail in them (the elements are embedded at build time; the package needs a
# release binary first) that a raw command gets wrong once and then nobody
# remembers why the panel did not change.

CARGO ?= cargo
PORT ?= 8083
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
ARCH ?= amd64
# The same version as PEP 440 spells it, for the client's wheel: 0.3.0-alpha1 is 0.3.0a1.
PY_VERSION ?= $(shell echo '$(VERSION)' | sed -e 's/-alpha/a/' -e 's/-beta/b/' -e 's/-rc/rc/')

.PHONY: build test check proto check-proto web check-web client check-schema check-text run dev schema package wheel print-version clean help firmware-proto test-firmware firmware

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
	@# Not check-web: it needs npm and a network on first run, and this target
	@# has to work on a rig. CI runs `make check-web` as its own step.

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
	  proto/mousewheeld/v1/*.proto proto/mousewheeld/link/v1/*.proto
	@rm -rf target/proto-check
	@$(CARGO) run --quiet --manifest-path tools/protogen/Cargo.toml -- target/proto-check
	@# Against a fresh generation rather than against git: a *new* generated
	@# file is untracked, and `git diff` says nothing about an untracked file.
	@# `mod.rs` beside them is hand-written and is not generated into the
	@# scratch tree, so it is not compared — a check that flagged it would fail
	@# every time somebody wrote a comment.
	@diff -r --exclude=mod.rs target/proto-check daemon/src/wire || { \
	  echo "daemon/src/wire/ is not what proto/ produces — the interface changed."; \
	  echo "run 'make proto' and commit the result with the change that caused it."; \
	  exit 1; \
	}

# **The browser's protobuf client is generated and committed**, like
# daemon/src/wire/ and for the same reason: rust-embed reads client/web/elements/
# at compile time, so a bundle produced during `cargo build` would make npm a
# build dependency of the daemon — on every rig, for every release. It is not.
# `npm ci` installs exactly what package-lock.json pins, so the bundle is
# reproducible; `make check-web` is what holds it to the proto.
web: ## Regenerate client/web/elements/daemon_api_client.js from proto/
	@cd client/web && npm ci --silent --no-audit --no-fund && node build_daemon_api_client.mjs

check-web: ## Fail if the committed browser client is not what proto/ produces
	@cp client/web/elements/daemon_api_client.js target/web-check.js 2>/dev/null || true
	@$(MAKE) --no-print-directory web
	@diff -q target/web-check.js client/web/elements/daemon_api_client.js || { \
	  echo "client/web/elements/daemon_api_client.js was not what proto/ produces:"; \
	  diff target/web-check.js client/web/elements/daemon_api_client.js | head -20; \
	  echo "it has been regenerated — commit it with the change that caused it."; \
	  exit 1; \
	}

# The Python client is its own project with its own Makefile, and is not part
# of `check` for the same reason `check-web` is not: it needs uv, and a network
# the first time. Its own `check` regenerates its stubs from proto/, typechecks,
# and runs both suites — the second of which starts a simulated daemon.
client: ## The Python client's checks: its stubs, ty, and both test suites
	@$(MAKE) --no-print-directory -C client/python check

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
	cd packaging && VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f nfpm.yaml -p deb -t ../dist/
	cd packaging && VERSION=$(VERSION) ARCH=$(ARCH) nfpm package -f nfpm.yaml -p rpm -t ../dist/

print-version: ## The version a build of this checkout carries
	@echo $(VERSION)

# The client's pyproject carries the 0.0.0 sentinel, so the release version is
# stamped into a copy of it here rather than hand-edited in the tree: the
# workspace version in Cargo.toml is the one place a release number is written.
wheel: ## The Python client's wheel, stamped with the workspace version
	@rm -rf target/wheel && mkdir -p target/wheel dist
	@tar -C client/python --exclude=.venv --exclude=build --exclude=dist \
	  --exclude=__pycache__ --exclude=.pytest_cache -cf - . | tar -C target/wheel -xf -
	cd target/wheel && uv version --frozen $(PY_VERSION) >/dev/null && \
	  uv build --wheel --out-dir $(CURDIR)/dist

clean:
	$(CARGO) clean
	rm -rf dist

# The link proto is generated for the board with nanopb and committed, like
# daemon/src/wire/: a firmware build needs no generator, and a change to the
# link shows up as a diff. The generator version must match the runtime in
# firmware/third_party/nanopb (0.4.9.1).
firmware-proto: ## Regenerate firmware/core/proto/ from proto/mousewheeld/link/v1/
	uvx --from nanopb==0.4.9.1 nanopb_generator -I proto -D firmware/core/proto \
	  -f firmware/core/proto/link.options proto/mousewheeld/link/v1/link.proto

test-firmware: ## The firmware core's unit tests, on the host, under ASan and UBSan
	cmake -S firmware -B firmware/build -DMOUSEWHEELD_SANITIZE=ON
	cmake --build firmware/build -j
	ctest --test-dir firmware/build --output-on-failure

firmware: ## The ESP32 image (firmware/.pio/build/esp32/firmware.bin)
	cd firmware && MOUSEWHEELD_FIRMWARE_VERSION=$(VERSION) uvx --with pip platformio run -e esp32
