# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
.PHONY: help test test-ir lock-ir test-formal test-formal-native test-formal-parity check-formal-evidence check-embedded-build smoke-live bench-budgets test-licenses test-secrets
TARGET_DIR ?= target

test:
	python3 -I scripts/test.py
	$(MAKE) test-gui
	$(MAKE) test-app-packaging
	$(MAKE) smoke-app

help:
	@printf '%s\n' \
	  'make verify-clean-start        Private Codex defaults, CLI/seat, native GUI and Bridge checks' \
	  'make verify-harness-onboarding  Private discovery, setup, installer and user Markdown checks' \
	  'make verify-harness-attachments  Focused private composer, clipboard, ACP and ownership checks' \
	  'make test                       Full unattended unit, integration, formal, GUI and packaging checks' \
	  'make test-licenses              Offline suite policy, source credit and negative controls' \
	  'make check-license-packages     Check actual Cargo package contents without compiling' \
	  'make test-formal                Native Yarn/formal checks and broken controls' \
	  'make test-formal-parity         Native/Java TLC graph and verdict parity' \
	  'make land                       Signed rebase, focused gates, source-bound evidence and main push' \
	  'make test-python-execute         Real CPython, commands, current-context MCP and durable recovery' \
	  'make verify-gui-polish          Focused MCP output, prompt projection, loader and changed model checks' \
	  'make verify-quiet-code-cards    Focused Python/title/MCP geometry, native raster and embedded lint checks' \
	  'make check-embedded-lints       Strict embedded harness library lint compilation only' \
	  'make check-embedded-build       Strict embedded lint and default/ACP/standalone compile checks' \
	  'make app / app-unsigned         Release app, Apple identity / ad-hoc signed' \
	  'make install                    Apple-signed app into ~/Applications' \
	  'make install-unsigned           Ad-hoc app into target/install-unsigned' \
	  'make install-cli                Link the installed binary if ~/.local/bin exists; no sudo' \
	  'make test-app-packaging         Private launcher, signing and repeated-install checks' \
	  'make smoke-app                  Private replay and hidden native-window startup' \
	  'make test-app-startup-control   Reject the missing title-bar image, then restore' \
	  'make release-dry / release      Signed archive / publish verified main' \
	  'Overrides: TARGET_DIR=... INSTALL_DIR=... UNSIGNED_INSTALL_DIR=...'

.PHONY: check-embedded-lints
check-embedded-lints:
	cargo clippy --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --no-default-features --features embedded --lib --no-deps -- -D warnings

check-embedded-build:
	cargo clippy --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --no-default-features --features embedded --lib -- -D warnings
	cargo check --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --lib
	cargo check --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --features acp,standalone --lib --bin bboxy450-harness

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
	python3 -I scripts/suite-licenses.py
	python3 -I scripts/test-suite-licenses.py

.PHONY: check-licenses check-license-packages
check-licenses:
	python3 -I scripts/checks.py headers
	python3 -I scripts/suite-licenses.py

check-license-packages: check-licenses
	python3 -I scripts/suite-licenses.py --packages

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
INSTALL_DIR ?= $(HOME)/Applications
UNSIGNED_INSTALL_DIR ?= $(TARGET_DIR)/install-unsigned
BUNDLE := $(TARGET_DIR)/$(APP).app
DIST_DIR := $(TARGET_DIR)/release-artifacts
APP_VERSION = $(shell python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')
ARCHIVE = $(DIST_DIR)/$(APP)-$(APP_VERSION)-macos-arm64.zip
SIGNING_IDENTITY := 3943AF130A015B36824B7FBC86AA672E7B25F3C4
GPG_KEY := 9B07E849480C68328CA20684844ECC0587DE7AB0
REQUIREMENT := identifier "com.bboxy450.harness" and anchor apple generic and certificate leaf[subject.OU] = "UQB3368A84"

.PHONY: test-app-startup-control app-files app app-unsigned install install-unsigned install-cli smoke-app release-dry release test-app-packaging
app-files: check-licenses
	@test "$$(uname -s)" = Darwin || { echo 'macOS is required' >&2; exit 2; }
	cargo build --locked --release --features standalone --target-dir '$(TARGET_DIR)'
	@set -eu; rm -rf '$(BUNDLE)/Contents'; \
	mkdir -p '$(BUNDLE)/Contents/MacOS' '$(BUNDLE)/Contents/Resources'; \
	cp packaging/macos/Info.plist '$(BUNDLE)/Contents/Info.plist'; \
	cp '$(TARGET_DIR)/release/$(APP)' '$(BUNDLE)/Contents/MacOS/$(APP)'; \
	cp packaging/macos/launcher.sh '$(BUNDLE)/Contents/MacOS/launcher'; \
	chmod 755 '$(BUNDLE)/Contents/MacOS/launcher'; \
	cp LICENSE NOTICE ATTRIBUTION.md packaging/macos/bboxy.icns '$(BUNDLE)/Contents/Resources/'; \
	plutil -replace CFBundleShortVersionString -string '$(APP_VERSION)' '$(BUNDLE)/Contents/Info.plist'; \
	plutil -replace CFBundleVersion -string "$$(git rev-list --count HEAD)" '$(BUNDLE)/Contents/Info.plist'; \
	revision=$$(git rev-parse HEAD); \
	git diff --quiet HEAD -- src tests scripts packaging Cargo.toml Cargo.lock Makefile LICENSE NOTICE ATTRIBUTION.md licensing.json README.md || revision="$$revision-dirty"; \
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

# Adapted from bboxy450-term Makefile install_bundle:
# https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
# Keep the outer .app inode for Dock/Finder aliases and retain previous Contents.
define install_bundle
set -eu; mkdir -p '$(1)'; \
stage=$$(mktemp -d '$(1)/.$(APP)-install.XXXXXX'); \
ditto '$(BUNDLE)' "$$stage/new.app"; codesign --verify --strict "$$stage/new.app"; \
destination='$(1)/$(APP).app'; mkdir -p "$$destination"; \
if [ -d "$$destination/Contents" ]; then mv "$$destination/Contents" "$$stage/previous-Contents"; fi; \
if ! mv "$$stage/new.app/Contents" "$$destination/Contents"; then \
  if [ -d "$$stage/previous-Contents" ]; then mv "$$stage/previous-Contents" "$$destination/Contents"; fi; exit 1; \
fi; \
codesign --verify --strict --verbose=2 "$$destination"; \
printf 'Installed: %s\nPrevious contents, if any: %s\n' "$$destination" "$$stage/previous-Contents"
endef

install: app
	@$(call install_bundle,$(INSTALL_DIR))

install-unsigned: app-unsigned
	@$(call install_bundle,$(UNSIGNED_INSTALL_DIR))

install-cli:
	@set -eu; if [ -d '$(HOME)/.local/bin' ]; then \
	  binary="$$(cd '$(INSTALL_DIR)' && pwd)/$(APP).app/Contents/MacOS/$(APP)"; \
	  test -x "$$binary"; \
	  ln -sfn "$$binary" '$(HOME)/.local/bin/$(APP)'; \
	  printf 'Linked: %s\n' '$(HOME)/.local/bin/$(APP)'; \
	else printf '%s\n' 'Skipped CLI link: ~/.local/bin does not exist.'; fi

test-app-packaging:
	python3 -I tests/app_packaging.py

test-app-startup-control:
	python3 -I scripts/test-startup-control.py

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

# Bboxy log IR v1: only add, never change or remove (.bboxy/specs/log-ir-v1.md); also run by `make test`.
test-ir:
	python3 -I scripts/ir-compat.py check --require-current
	python3 -I scripts/ir-compat.py control

# Refresh the IR lock after an addition; refused unless the schema is additive over the old lock.
lock-ir:
	UPDATE_GOLDENS=1 cargo test --offline --locked --test protocol_golden schema_and_golden_transcripts -- --exact
	python3 -I scripts/ir-compat.py lock

.PHONY: verify-ir-defaults
verify-ir-defaults:
	cargo test --offline --locked --test acp_controls acp_controls_missing_fields_reject_before_wire -- --exact
	cargo test --offline --locked --lib image_intake_proof_rejects_durable_replay_old_epoch_and_reconnect_challenge
	cargo test --offline --locked --test ir_scan
	$(MAKE) test-ir

.PHONY: verify-gui-polish
verify-gui-polish:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/verify-gui-polish.py

.PHONY: verify-profiles-pending
verify-profiles-pending:
	cargo test --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --features embedded --lib gui_polish_pending -- --nocapture
	PROFILES_PENDING_CAPTURE_DIR='$(abspath $(TARGET_DIR))/profiles-pending/captures' cargo test --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --features embedded --test gui_render profiles_pending -- --nocapture
	$(MAKE) check-embedded-lints
	cargo check --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' --lib

.PHONY: verify-acp-controls
verify-acp-controls:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/test-acp-controls.py

.PHONY: test-acp-steering test-acp-steering-formal
test-rich-scroll:
	cargo test --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features embedded --lib spring_
	cargo test --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features embedded --test gui_scroll

test-rich-scroll-formal:
	SCROLL_FORMAL_OUT='$(abspath $(TARGET_DIR))/scroll/formal' cargo test --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --test formal_models scroll_native
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/check-scroll-models.py

.PHONY: test-rich-scroll test-rich-scroll-formal
test-acp-steering:
	cargo test --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features acp-agent --test acp_agent steering_ -- --test-threads=2

test-acp-steering-formal:
	ACP_STEERING_FORMAL_OUT='$(abspath $(TARGET_DIR))/acp-steering/formal' cargo test --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --test formal_models acp_steering_native
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/check-acp-steering-models.py

.PHONY: verify-harness-card-bounds
verify-harness-card-bounds:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/verify-harness-card-bounds.py

.PHONY: verify-quiet-code-cards
verify-quiet-code-cards:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/verify-quiet-code-cards.py

.PHONY: verify-harness-attachments
verify-harness-attachments:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/verify-harness-attachments.py

.PHONY: verify-harness-onboarding
verify-harness-onboarding:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/verify-harness-onboarding.py

.PHONY: verify-onboarding-routing
verify-onboarding-routing:
	cargo nextest run --offline --locked --jobs 2 --target-dir '$(TARGET_DIR)' -p bboxy450-harness --features standalone --test harness_onboarding -E 'test(onboarding_loader_welcome_routing)'

.PHONY: verify-clean-start
verify-clean-start:
	TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/verify-clean-start.py

.PHONY: prepare-dream-c1-worker test-dream-c1 test-dream-c1-integration
prepare-dream-c1-worker:
	python3 -I tests/fixtures/dream-bootstrap/check.py
	cargo build --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --bin bboxy450-harness
	mkdir -p '$(TARGET_DIR)/dream-c1-native-profile'
	cp '$(TARGET_DIR)/release/bboxy450-harness' '$(TARGET_DIR)/dream-c1-native-profile/worker'
	printf '%s\n' 'dream-bootstrap/1' > '$(TARGET_DIR)/dream-c1-native-profile/bootstrap-support.txt'
	git rev-parse HEAD > '$(TARGET_DIR)/dream-c1-native-profile/source-commit.txt'
	rustc -Vv > '$(TARGET_DIR)/dream-c1-native-profile/compiler.txt'
	shasum -a 256 src/main.rs src/dream/mod.rs src/dream/c1.rs src/dream/bootstrap.rs tests/fixtures/dream-bootstrap/vectors.json tests/dream_c1.rs Cargo.toml Cargo.lock '$(TARGET_DIR)/dream-c1-native-profile/worker' > '$(TARGET_DIR)/dream-c1-native-profile/hashes.txt'
	python3 -I -c 'import hashlib,pathlib,shutil,sys; p=pathlib.Path(sys.argv[1]); h=hashlib.sha256(p.read_bytes()).hexdigest(); d=p.with_name("worker-"+h); shutil.copy2(p,d); print("approved bootstrap artifact",d,h)' '$(TARGET_DIR)/dream-c1-native-profile/worker'

test-dream-c1-integration: prepare-dream-c1-worker
	DREAM_C1_RUNNER='$(abspath $(TARGET_DIR))/dream-c1-native-profile/worker' cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features test-fixtures --test dream_c1 $(DREAM_C1_TEST_FILTER) -- --test-threads=1 --nocapture

test-dream-c1: prepare-dream-c1-worker
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --lib dream::bootstrap:: -- --test-threads=1
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --lib dream::c1:: -- --test-threads=1
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --lib dream::fingerprint:: -- --test-threads=1
	DREAM_C1_RUNNER='$(abspath $(TARGET_DIR))/dream-c1-native-profile/worker' cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --test dream_c1 --test dream_replay --test seat_process --test acp_launch_context -- --test-threads=1 --nocapture
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features test-fixtures --test dream_c1 bootstrap_earliest_gate_bypass_control -- --test-threads=1 --nocapture
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features test-fixtures --test seat_process dream_c1_retained -- --test-threads=1 --nocapture
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features test-fixtures --lib frontends::headless::dream_c1_fixture_environment_tests:: -- --test-threads=1
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --lib mcp::source_tests:: -- --test-threads=1
	cargo test --offline --locked --release --jobs 2 --target-dir '$(TARGET_DIR)' --features test-fixtures --test mcp_client -- --test-threads=1 --nocapture

.PHONY: test-python-execute test-python-execute-boundaries check-python-execute
check-python-execute:
	@set -eu; if [ -f ../../scripts/cache-env.sh ]; then . ../../scripts/cache-env.sh; fi; cargo check --offline --locked --jobs 2 -p bboxy450-harness --features acp-agent --lib --target-dir '$(TARGET_DIR)'
test-python-execute:
	@set -eu; if [ -f ../../scripts/cache-env.sh ]; then . ../../scripts/cache-env.sh; fi; cargo nextest run --offline --locked --jobs 2 -p bboxy450-harness --features acp-agent --lib --target-dir '$(TARGET_DIR)' -E 'test(python_execute::tests)'
test-python-execute-boundaries: test-python-execute
	@set -eu; if [ -f ../../scripts/cache-env.sh ]; then . ../../scripts/cache-env.sh; fi; cargo nextest run --offline --locked --jobs 2 -p bboxy450-harness --features acp-agent,test-fixtures --test mcp_client --test acp_launch_context --target-dir '$(TARGET_DIR)'

.PHONY: land
land:
	@set -eu; if [ -f ../../scripts/cache-env.sh ]; then . ../../scripts/cache-env.sh; fi; TARGET_DIR='$(abspath $(TARGET_DIR))' python3 -I scripts/land.py
