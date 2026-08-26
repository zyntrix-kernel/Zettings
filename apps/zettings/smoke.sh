#!/bin/bash
set -u
cd "$(dirname "$0")"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export QT_QPA_PLATFORM="${QT_QPA_PLATFORM:-offscreen}"
timeout "${SMOKE_SECONDS:-4}" ./zettings > /tmp/zettings-smoke.log 2>&1
EXIT=$?
if [ "$EXIT" -eq 124 ]; then
    echo "SMOKE PASS: ran ${SMOKE_SECONDS:-4}s until timeout (event loop healthy)"
    exit 0
fi
echo "SMOKE FAIL: exited early with $EXIT"
grep -viE 'libEGL|MESA|swrast|llvmpipe|EGL' /tmp/zettings-smoke.log | head -20
exit 1
