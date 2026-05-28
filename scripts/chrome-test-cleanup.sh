#!/usr/bin/env bash
# Clean up leaked headless Chrome instances and stale profile locks left by
# interrupted chrome-mode crawl tests. Run this before the test suite if chrome
# tests start hanging or reporting "channel closed".
#
# Only targets the chromiumoxide-runner profile, never a real browser (which
# uses ~/.config/google-chrome), so it is safe to run on a dev machine.
set -u

pkill -9 -f "chromiumoxide-runner" 2>/dev/null || true
rm -f /tmp/chromiumoxide-runner/Singleton* 2>/dev/null || true

echo "chrome test cleanup done"
