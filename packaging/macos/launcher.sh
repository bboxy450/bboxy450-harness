#!/bin/sh
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
set -eu
macos_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
if [ "$#" -eq 0 ]; then
    # Match the seat/headless default-profile precedence without reading keys.
    profile=${BBOXY450_HARNESS_PROFILE:-}
    if [ -z "$(printf '%s' "$profile" | tr -d '[:space:]')" ]; then
        profile=
        if [ -r "$HOME/.bboxy450-harness/default-profile" ]; then
            profile=$(sed 's/^[[:space:]]*//;s/[[:space:]]*$//' "$HOME/.bboxy450-harness/default-profile")
        fi
    fi
    set -- gui --cwd "$HOME" --profile "${profile:-nemotron}"
fi
exec "$macos_dir/bboxy450-harness" "$@"
