# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""The real face must learn its file value; an old-env-only reader must leak."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "src/capability.rs"
LOGS = ROOT / "target/capability-control"
TEST = "processes::capability_file_value_is_redacted_in_real_headless_tool_and_log"


def main():
    original = SOURCE.read_bytes()
    text = original.decode()
    start = text.index('    let path = std::env::var_os("BBOXY_CAPABILITY_FILE");')
    end = text.index("    match result {", start)
    # All faces share this bootstrap. Restore the old raw environment reader;
    # keep validators referenced so compilation cannot masquerade as the red test.
    broken = text[:start] + '''    let _reader = read;
    let result = std::env::var("BBOXY_CAPABILITY")
        .map(Zeroizing::new).map_err(|_| Refusal::MissingFile);
''' + text[end:]
    env = {key: value for key, value in os.environ.items() if not key.startswith("BBOXY_")}
    command = ["cargo", "test", "--offline", "--locked", "--features", "acp,test-fixtures",
               "--test", "capability_delivery", TEST, "--", "--exact", "--nocapture"]
    LOGS.mkdir(parents=True, exist_ok=True)
    try:
        SOURCE.write_text(broken)
        with (LOGS / "legacy-reader-red.log").open("wb") as output:
            red = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    finally:
        SOURCE.write_bytes(original)
    assert SOURCE.read_bytes() == original, "capability reader source was not restored"
    log = (LOGS / "legacy-reader-red.log").read_text()
    assert red.returncode == 101 and "running 1 test" in log and "test result: FAILED" in log
    assert "CapabilityFileValueIsRedacted" in log, "red control did not fail at the leak assertion"
    with (LOGS / "file-reader-green.log").open("wb") as output:
        green = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    assert green.returncode == 0, "restored file reader did not pass"
    receipt = dict(command=command, assertion="CapabilityFileValueIsRedacted",
                   red_exit=red.returncode, green_exit=green.returncode,
                   restored_source_sha256=hashlib.sha256(original).hexdigest(), operator_input=False)
    (LOGS / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    main()
