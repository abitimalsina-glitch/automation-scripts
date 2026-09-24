#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYTHON="$(command -v python3)"

if [ -z "$PYTHON" ]; then
    echo "Error: Python 3 is not installed."
    exit 1
fi

"$PYTHON" "$SCRIPT_DIR/finder.py"
