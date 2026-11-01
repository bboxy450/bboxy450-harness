# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Measurements never decide the unattended structural gate."""
import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
def main():
    load = os.getloadavg()[0]
    ncpu = os.cpu_count() or 1
    record = {'at': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'load_1m': load, 'ncpu': ncpu}
    if load > ncpu:
        record.update(outcome='UNMEASURED', exit_code=0)
        print(f'UNMEASURED: load {load:.2f} exceeds {ncpu} CPUs', flush=True)
    else:
        command = ['cargo', 'test', '--offline', '--locked', '--test', 'rt_budget', 'ten_thousand_file_grep_leaves_io_ticker_responsive', '--', '--ignored', '--exact', '--nocapture']
        result = subprocess.run(command, cwd=ROOT)
        record.update(outcome='PASS' if result.returncode == 0 else 'FAIL', command=command, exit_code=result.returncode)
        print(record['outcome'], flush=True)
    path = ROOT / '.bboxy/runs/bench-budgets-current.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2) + '\n')
    return record['exit_code']
if __name__ == '__main__': sys.exit(main())
