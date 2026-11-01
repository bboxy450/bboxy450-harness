# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Portable test-only equivalent of setsid sleep 30; inherits both output pipes."""
import os
from pathlib import Path
import sys
os.setsid()
Path(sys.argv[1]).write_text(str(os.getpid()))
os.execl('/bin/sleep','sleep','30')
