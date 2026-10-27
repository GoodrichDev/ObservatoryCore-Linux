CARGO ?= cargo
CARGO_TARGET_DIR ?= target
PREFIX ?= $(HOME)/.local
DESTDIR ?=
DIST_DIR ?= dist
ARGS ?=

.PHONY: all build run test check fmt package install clean legacy legacy-winesetup

all: build

build:
	$(CARGO) build --release --locked -p observatory-desktop

run:
	$(CARGO) run --release --locked -p observatory-desktop -- $(ARGS)

test:
	$(CARGO) test --workspace --locked

check:
	$(CARGO) fmt --all --check
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

fmt:
	$(CARGO) fmt --all

package: build
	sh native/scripts/package.sh --binary "$(CARGO_TARGET_DIR)/release/observatory-core" --out "$(DIST_DIR)"

install: build
	@set -eu; stage=$$(mktemp -d "$${TMPDIR:-/tmp}/observatory-install.XXXXXXXX"); \
	trap 'rm -rf -- "$$stage"' EXIT HUP INT TERM; \
	sh native/scripts/package.sh --tar-only --binary "$(CARGO_TARGET_DIR)/release/observatory-core" --out "$$stage/packages"; \
	tar -xzf "$$stage"/packages/observatory-core-*-linux.tar.gz -C "$$stage"; \
	sh "$$stage"/observatory-core-*/install.sh --prefix "$(PREFIX)" --destdir "$(DESTDIR)"

clean:
	$(CARGO) clean

# Explicit opt-in to the retained .NET/Wine build, for source comparison only.
legacy:
	$(MAKE) -f Makefile.wine all

legacy-winesetup:
	$(MAKE) -f Makefile.wine winesetup
