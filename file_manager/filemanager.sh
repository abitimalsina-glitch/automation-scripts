#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYTHON="$(command -v python3)"

if [ -z "$PYTHON" ]; then
    echo "Error: Python 3 is not installed."
    exit 1
fi

if [ $# -ne 1 ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

TARGET="$1"

if [[ "$TARGET" != /* && "$TARGET" != ~* ]]; then
    TARGET="$HOME/$TARGET"
fi

if [ ! -d "$TARGET" ]; then
    echo "Error: Directory '$TARGET' not found."
    exit 1
fi

TARGET_DIR="$(cd "$TARGET" && pwd)"

"$PYTHON" "$SCRIPT_DIR/manager.py" "$TARGET_DIR"
