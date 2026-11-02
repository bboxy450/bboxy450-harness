# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
.PHONY: test test-formal test-formal-native test-formal-parity check-formal-evidence smoke-live bench-budgets test-licenses test-secrets
TARGET_DIR ?= target

test:
	python3 -I scripts/test.py
	$(MAKE) test-gui
	$(MAKE) test-app-packaging

test-formal-native:
	FORMAL_OUT='$(abspath $(TARGET_DIR))/formal/models' cargo test --offline --locked --test formal_models --test formal_nonvacuity --test reactor_projection
	python3 -I scripts/formal.py --native --target '$(abspath $(TARGET_DIR))/formal'

test-formal: test-formal-native

test-formal-parity: test-formal-native
	python3 -I scripts/test-formal.py
	python3 -I scripts/formal.py --target '$(abspath $(TARGET_DIR))/formal'

check-formal-evidence:
	python3 -I scripts/formal.py --target '$(abspath $(TARGET_DIR))/formal' --verify-evidence

smoke-live:
	python3 -I scripts/smoke-live.py

bench-budgets:
	python3 -I scripts/bench-budgets.py

test-licenses:
	python3 -I scripts/checks.py headers

test-secrets:
	python3 -I scripts/checks.py secrets

.PHONY: test-gui test-gui-smoke test-gui-lint
test-gui:
	python3 -I tests/gui_gate.py

test-gui-lint:
	CLIPPY_CONF_DIR='$(CURDIR)/src/frontends/gui/lint' cargo clippy --offline --locked --all-features --lib --bin bboxy450-harness -- -D warnings

test-gui-smoke:
	cargo run --offline --locked --features standalone -- gui --replay tests/fixtures/gui/rich-all-items.jsonl --headless --screenshot .bboxy/runs/gui-smoke.png

.PHONY: bench-budgets bench-gui-budget
bench-budgets: bench-gui-budget

bench-gui-budget:
	cargo test --offline --locked --features standalone --test gui_bench -- --ignored --nocapture

# Adapted from bboxy450-term Makefile bundle/release recipes:
# https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
APP := bboxy450-harness
BUNDLE := $(TARGET_DIR)/Bboxy Harness.app
DIST_DIR := $(TARGET_DIR)/release-artifacts
APP_VERSION = $(shell python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')
ARCHIVE = $(DIST_DIR)/$(APP)-$(APP_VERSION)-macos-arm64.zip
SIGNING_IDENTITY := 3943AF130A015B36824B7FBC86AA672E7B25F3C4
GPG_KEY := 9B07E849480C68328CA20684844ECC0587DE7AB0
REQUIREMENT := identifier "com.bboxy450.harness" and anchor apple generic and certificate leaf[subject.OU] = "UQB3368A84"

.PHONY: app-files app app-unsigned smoke-app release-dry release test-app-packaging
app-files:
	@test "$$(uname -s)" = Darwin || { echo 'macOS is required' >&2; exit 2; }
	cargo build --locked --release --features standalone --target-dir '$(TARGET_DIR)'
	@set -eu; rm -rf '$(BUNDLE)/Contents'; \
	mkdir -p '$(BUNDLE)/Contents/MacOS' '$(BUNDLE)/Contents/Resources'; \
	cp packaging/macos/Info.plist '$(BUNDLE)/Contents/Info.plist'; \
	cp '$(TARGET_DIR)/release/$(APP)' '$(BUNDLE)/Contents/MacOS/$(APP)'; \
	cp packaging/macos/launcher.sh '$(BUNDLE)/Contents/MacOS/launcher'; \
	chmod 755 '$(BUNDLE)/Contents/MacOS/launcher'; \
	cp LICENSE ATTRIBUTION.md packaging/macos/bboxy.icns '$(BUNDLE)/Contents/Resources/'; \
	plutil -replace CFBundleShortVersionString -string '$(APP_VERSION)' '$(BUNDLE)/Contents/Info.plist'; \
	plutil -replace CFBundleVersion -string "$$(git rev-list --count HEAD)" '$(BUNDLE)/Contents/Info.plist'; \
	revision=$$(git rev-parse HEAD); \
	git diff --quiet HEAD -- src tests scripts packaging Cargo.toml Cargo.lock Makefile LICENSE ATTRIBUTION.md README.md || revision="$$revision-dirty"; \
	plutil -insert BBOXYSourceRevision -string "$$revision" '$(BUNDLE)/Contents/Info.plist'; \
	plutil -lint '$(BUNDLE)/Contents/Info.plist'

app: app-files
	@set -eu; security find-identity -v -p codesigning | grep -F '$(SIGNING_IDENTITY)' >/dev/null || { echo 'Approved Apple Development signing identity unavailable' >&2; exit 2; }; \
	codesign --force --sign '$(SIGNING_IDENTITY)' --options runtime --timestamp=none '$(BUNDLE)/Contents/MacOS/$(APP)'; \
	codesign --force --sign '$(SIGNING_IDENTITY)' --requirements '=designated => $(REQUIREMENT)' --options runtime --timestamp=none '$(BUNDLE)'; \
	codesign --verify --strict --verbose=2 '$(BUNDLE)'; printf '%s\n' '$(BUNDLE)'

app-unsigned: app-files
	@set -eu; codesign --force --sign - --timestamp=none '$(BUNDLE)/Contents/MacOS/$(APP)'; \
	codesign --force --sign - --timestamp=none '$(BUNDLE)'; \
	codesign --verify --strict '$(BUNDLE)'; \
	printf '%s\n' 'Test bundle: ad-hoc signature only; no Apple identity or Keychain access.'

test-app-packaging:
	python3 -I tests/app_packaging.py

smoke-app: app-unsigned
	python3 -I scripts/smoke-app.py '$(BUNDLE)' '$(TARGET_DIR)/app-smoke'

# Dry runs use ad-hoc signing, so no Apple private key or Keychain is consulted.
# The publishing target rebuilds and signs with the approved Apple identity.
define archive_app
set -eu; test "$$(uname -m)" = arm64 || { echo 'arm64 is required for this release artifact' >&2; exit 2; }; \
mkdir -p '$(DIST_DIR)'; \
ditto -c -k --sequesterRsrc --keepParent '$(BUNDLE)' '$(ARCHIVE)'; \
(cd '$(DIST_DIR)' && shasum -a 256 '$(notdir $(ARCHIVE))' >'$(notdir $(ARCHIVE)).sha256'); \
gpg --batch --yes --local-user '$(GPG_KEY)' --armor --detach-sign '$(ARCHIVE)'; \
gpg --batch --verify '$(ARCHIVE).asc' '$(ARCHIVE)'; \
(cd '$(DIST_DIR)' && shasum -a 256 -c '$(notdir $(ARCHIVE)).sha256'); \
printf 'Signed release artifacts: %s{,.sha256,.asc}\n' '$(ARCHIVE)'
endef

release-dry: app-unsigned
	@$(archive_app)

release: app
	@set -eu; test "$$(gh api user --jq .login)" = bboxy450; \
	test -z "$$(git status --porcelain)"; test "$$(git branch --show-current)" = main; \
	git verify-commit HEAD
	@$(archive_app)
	@set -eu; tag='v$(APP_VERSION)'; \
	git tag -s "$$tag" -m '$(APP) $(APP_VERSION)'; \
	GIT_TERMINAL_PROMPT=0 git push --follow-tags; \
	gh release create "$$tag" --repo bboxy450/bboxy450-harness '$(ARCHIVE)' '$(ARCHIVE).sha256' '$(ARCHIVE).asc' --title '$(APP) $(APP_VERSION)' --notes "Source: https://github.com/bboxy450/bboxy450-harness/commit/$$(git rev-parse HEAD)"
