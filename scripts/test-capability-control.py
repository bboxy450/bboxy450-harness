# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Every face must load the file on its real startup path; mutations must fail by assertion."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
LOGS = ROOT / "target/capability-control"
HEADLESS = "src/frontends/headless.rs"
BRIDGE = "src/frontends/gui/bridge.rs"
# Shared headless code routes three distinct faces; each mutation omits only its
# selected face. GUI smoke and Bridge::replay are separate startup paths.
CASES = [
    ("legacy-reader", "src/capability.rs", "processes::capability_file_value_is_redacted_in_real_headless_tool_and_log", "HeadlessCapabilityRegistered"),
    ("headless", HEADLESS, "processes::capability_file_value_is_redacted_in_real_headless_tool_and_log", "HeadlessCapabilityRegistered"),
    ("seat", HEADLESS, "processes::seat_loads_capability_file_before_tool_output", "SeatCapabilityRegistered"),
    ("acp", HEADLESS, "processes::acp_loads_capability_file_before_tool_output", "AcpCapabilityRegistered"),
    ("gui-replay", BRIDGE, "gui::gui_replay_bridge_loads_capability_and_only_notices_supplied_invalid_files", "GuiReplayCapabilityRegistered"),
    ("gui-live", BRIDGE, "gui::live_gui_registers_capability_before_host_starts", "GuiLiveCapabilityRegisteredBeforeHost"),
    ("gui-smoke", "src/frontends/gui/smoke.rs", "gui::gui_smoke_loads_file_and_renders_the_same_frame_as_explicit_redaction", "GuiSmokeRegistersCapabilityBeforeRendering"),
    ("smoke", "src/frontends/smoke.rs", "processes::optional_live_smoke_checks_the_file_before_its_key_skip", "SmokeCapabilityCheckedBeforeKeySkip"),
]


def mutate(text, name):
    if name == "legacy-reader":
        start = text.index('    let Some(path) = std::env::var_os("BBOXY_CAPABILITY_FILE")')
        end = text.index("    match result {", start)
        # Keep validators referenced: compilation failure cannot count as red.
        return text[:start] + '''    let _reader = read;
    let result = std::env::var("BBOXY_CAPABILITY")
        .map(Zeroizing::new).map_err(|_| Refusal::MissingFile);
''' + text[end:]
    call = "crate::capability::register(&redactor).await;"
    if name in ("headless", "seat", "acp"):
        guard = {"headless": "seat || acp", "seat": "!seat", "acp": "!acp"}[name]
        assert text.count(call) == 1
        return text.replace(call, "if " + guard + " { " + call + " }")
    if name == "gui-replay":
        before, after = text.split("pub fn replay(&mut self, path: PathBuf)", 1)
        assert call in after
        return before + "pub fn replay(&mut self, path: PathBuf)" + after.replace(call, "// omitted replay registration", 1)
    if name == "gui-live":
        call = "crate::capability::register(&config.redactor).await;"
    assert text.count(call) == 1
    return text.replace(call, "// omitted face registration")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", action="append", choices=[case[0] for case in CASES])
    chosen = parser.parse_args().case
    env = {key: value for key, value in os.environ.items() if not key.startswith("BBOXY_")}
    LOGS.mkdir(parents=True, exist_ok=True)
    receipts = []
    for name, relative, test, assertion in CASES:
        if chosen and name not in chosen:
            continue
        source = ROOT / relative
        original = source.read_bytes()
        command = ["cargo", "test", "--offline", "--locked", "--features", "acp,standalone,test-fixtures",
                   "--test", "capability_delivery", test, "--", "--exact", "--nocapture"]
        red_log = LOGS / (name + "-red.log")
        green_log = LOGS / (name + "-green.log")
        try:
            source.write_text(mutate(original.decode(), name))
            with red_log.open("wb") as output:
                red = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
        finally:
            source.write_bytes(original)
        assert source.read_bytes() == original, "face source was not restored: " + name
        log = red_log.read_text()
        assert red.returncode == 101 and "running 1 test" in log and "test result: FAILED" in log, name + " did not run one failing test"
        assert assertion in log, name + " did not fail at " + assertion
        with green_log.open("wb") as output:
            green = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
        assert green.returncode == 0, name + " was not green after restoring its loader"
        receipt = dict(name=name, source=relative, command=command, assertion=assertion,
                       red_exit=red.returncode, green_exit=green.returncode,
                       restored_source_sha256=hashlib.sha256(original).hexdigest(), operator_input=False)
        receipts.append(receipt)
        print(json.dumps(receipt), flush=True)
    (LOGS / "receipt.json").write_text(json.dumps(receipts, indent=2) + "\n")


if __name__ == "__main__":
    main()
