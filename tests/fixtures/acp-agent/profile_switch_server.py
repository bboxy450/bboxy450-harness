# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Private server identity is process context, not a configured secret value."""
import os
from pathlib import Path
import runpy
import sys

os.environ["BBOXY_SEAT_ID"] = "private-profile-seat"
os.environ["BBOXY_GENERATION"] = "7"
sys.argv = [__file__, "server"]
runpy.run_path(str(Path(__file__).with_name("launch_parity.py")), run_name="__main__")
