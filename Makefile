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
