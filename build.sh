#!/usr/bin/env bash
set -e
echo "=================================================="
echo "          BUILDING VEYRON OS FROM SCRATCH         "
echo "=================================================="
if command -v python3 &>/dev/null; then
    python3 tools/build_os.py
else
    python tools/build_os.py
fi
